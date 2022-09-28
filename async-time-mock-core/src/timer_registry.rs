use crate::time_handler_guard::TimeHandlerGuard;
use crate::timer::{Timer, TimerListener};
use crate::{Instant, Interval};
use event_listener::Event;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::time::Duration;

#[derive(Default)]
pub struct TimerRegistry {
	current_time: RwLock<Duration>,
	timers_by_time: RwLock<TimersByTime>,
	any_timer_scheduled_signal: Event,
	advance_time_lock: async_lock::Mutex<()>,
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
	pub fn sleep_until(&self, until: Instant) -> impl Future<Output = TimeHandlerGuard> + Send + Sync + 'static {
		let timer = {
			let timers_by_time = self.timers_by_time.write().expect("RwLock was poisoned");
			let wakeup_time = until.into_duration();
			Self::schedule_timer(timers_by_time, wakeup_time)
		};
		self.any_timer_scheduled_signal.notify(1);

		timer.wait_until_triggered()
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
		Instant::new(*self.current_time.read().expect("RwLock was poisoned"))
	}
}

impl Debug for TimerRegistry {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let Self {
			current_time,
			timers_by_time: _,
			any_timer_scheduled_signal: _,
			advance_time_lock: _,
		} = self;
		formatter
			.debug_struct("TimerRegistry")
			.field("current_time", current_time)
			.finish_non_exhaustive()
	}
}
