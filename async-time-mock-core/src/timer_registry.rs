use crate::time_handler_guard::TimeHandlerGuard;
use crate::timeout::Timeout;
use crate::timer::{Timer, TimerListener};
use crate::{Instant, Interval};
use event_listener::Event;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::time::Duration;

pub struct TimerRegistry {
	id: u64,
	current_time: RwLock<Duration>,
	timers_by_time: RwLock<TimersByTime>,
	any_timer_scheduled_signal: Event,
	advance_time_lock: async_lock::Mutex<()>,
}

impl Default for TimerRegistry {
	fn default() -> Self {
		Self {
			id: Self::next_id(),
			current_time: Default::default(),
			timers_by_time: Default::default(),
			any_timer_scheduled_signal: Default::default(),
			advance_time_lock: Default::default(),
		}
	}
}

type TimersByTime = BTreeMap<Duration, VecDeque<Timer>>;

impl TimerRegistry {
	/// Schedules a timer to expire in "Duration", once expired, returns
	/// a TimeHandlerGuard that must be dropped only once the timer event has been fully processed
	/// (all sideeffects finished).
	///
	/// Roughly eqivalent to `async pub fn sleep(&self, duration: Duration) -> TimeHandlerGuard`.
	pub fn sleep(&self, duration: Duration) -> impl Future<Output = TimeHandlerGuard> + Send + Sync + 'static {
		assert!(!duration.is_zero(), "Sleeping for zero time is not allowed");

		let timer = {
			let timers_by_time = self.timers_by_time.write().expect("RwLock was poisoned");
			let wakeup_time = *self.current_time.read().expect("RwLock was poisoned") + duration;
			Self::schedule_timer(timers_by_time, wakeup_time)
		};
		self.any_timer_scheduled_signal.notify(1);

		timer.wait_until_triggered()
	}

	/// Schedules a timer to expire at "Instant", once expired, returns
	/// a TimeHandlerGuard that must be dropped only once the timer event has been fully processed
	/// (all sideeffects finished).
	///
	/// Roughly eqivalent to `async pub fn sleep_until(&self, until: Instant) -> TimeHandlerGuard`.
	///
	/// # Panics
	/// When `until` was created by a different instance of `TimerRegistry`.
	pub fn sleep_until(&self, until: Instant) -> impl Future<Output = TimeHandlerGuard> + Send + Sync + 'static {
		let timer = {
			let timers_by_time = self.timers_by_time.write().expect("RwLock was poisoned");
			let wakeup_time = until.into_duration(self.id);
			Self::schedule_timer(timers_by_time, wakeup_time)
		};
		self.any_timer_scheduled_signal.notify(1);

		timer.wait_until_triggered()
	}

	/// Combines a future with a `sleep` timer. If the future finishes before
	/// the timer has expired, returns the futures output. Otherwise returns
	/// `Elapsed` which contains a `TimeHandlerGuard` that must be dropped once the timeout has been fully processed
	/// (all sideeffects finished).
	///
	/// Roughly equivalent to `async pub fn timeout<F: Future>(&self, timeout: Duration, future: F) -> Result<F::Output, Elapsed>`
	pub fn timeout<F>(&self, timeout: Duration, future: F) -> Timeout<F>
	where
		F: Future,
	{
		Timeout::new(future, self.sleep(timeout))
	}

	/// Combines a future with a `sleep_until` timer. If the future finishes before
	/// the timer has expired, returns the futures output. Otherwise returns
	/// `Elapsed` which contains a `TimeHandlerGuard` that must be dropped once the timeout has been fully processed
	/// (all sideeffects finished).
	///
	/// Roughly equivalent to `async pub fn timeout_at<F: Future>(&self, at: Instant, future: F) -> Result<F::Output, Elapsed>`
	///
	/// # Panics
	/// When `at` was created by a different instance of `TimerRegistry`.
	pub fn timeout_at<F>(&self, at: Instant, future: F) -> Timeout<F>
	where
		F: Future,
	{
		Timeout::new(future, self.sleep_until(at))
	}

	pub fn interval(self: &Arc<Self>, period: Duration) -> Interval {
		Interval::new(self.clone(), self.now(), period)
	}

	pub fn interval_at(self: &Arc<Self>, start: Instant, period: Duration) -> Interval {
		Interval::new(self.clone(), start, period)
	}

	fn schedule_timer(mut timers_by_time: RwLockWriteGuard<'_, TimersByTime>, at: Duration) -> TimerListener {
		let (timer, listener) = Timer::new();
		timers_by_time.entry(at).or_insert_with(VecDeque::new).push_back(timer);
		listener
	}

	/// Advances test time by the given duration. Starts all scheduled timers that have expired
	/// at the new (advanced) point in time in the following order:
	/// 1. By time they are scheduled to run at
	/// 2. By the order they were scheduled
	///
	/// If no timer has been scheduled yet, waits until one is.
	/// Returns only once all started timers have finished processing.
	pub async fn advance_time(&self, by_duration: Duration) {
		let _guard = self.advance_time_lock.lock().await;

		let finished_time = *self.current_time.read().expect("RwLock was poisoned") + by_duration;

		if self.timers_by_time.read().expect("RwLock was poisoned").is_empty() {
			// If no timer has been scheduled yet, wait for one to be scheduled
			self.any_timer_scheduled_signal.listen().await;
		}

		loop {
			let timers_to_run = {
				let mut timers_by_time = self.timers_by_time.write().expect("RwLock was poisoned");
				match timers_by_time.keys().next() {
					Some(&key) if key <= finished_time => {
						let mut current_time = self.current_time.write().expect("RwLock was poisoned");
						*current_time = key.max(*current_time);
						timers_by_time
							.remove(&key)
							.unwrap_or_else(|| unreachable!("We just checked that it exists"))
					}
					_ => break,
				}
			};
			for timer in timers_to_run {
				let time_handler_finished = timer.trigger();
				time_handler_finished.wait().await;
			}
		}

		*self.current_time.write().expect("RwLock was poisoned") = finished_time;
	}

	/// Current test time, increases on every call to [`advance_time`].
	pub fn now(&self) -> Instant {
		Instant::new(*self.current_time.read().expect("RwLock was poisoned"), self.id)
	}

	fn next_id() -> u64 {
		static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

		ID_COUNTER.fetch_add(1, Ordering::Relaxed)
	}
}

impl Debug for TimerRegistry {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let Self {
			id,
			current_time,
			timers_by_time: _,
			any_timer_scheduled_signal: _,
			advance_time_lock: _,
		} = self;
		formatter
			.debug_struct("TimerRegistry")
			.field("id", id)
			.field("current_time", current_time)
			.finish_non_exhaustive()
	}
}
