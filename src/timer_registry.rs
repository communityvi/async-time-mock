use crate::time_handler_guard::TimeHandlerGuard;
use crate::timer::{Timer, TimerListener};
use event_listener::Event;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{RwLock, RwLockWriteGuard};
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
	pub async fn sleep(&self, duration: Duration) -> TimeHandlerGuard {
		assert!(!duration.is_zero(), "Sleeping for zero time is not allowed");

		let timer = {
			let timers_by_time = self.timers_by_time.write().expect("RwLock was poisoned");
			let wakeup_time = *self.current_time.read().expect("RwLock was poisoned") + duration;
			Self::schedule_timer(timers_by_time, wakeup_time)
		};
		self.any_timer_scheduled_signal.notify(1);

		timer.wait_until_triggered().await
	}

	fn schedule_timer(mut timers_by_time: RwLockWriteGuard<'_, TimersByTime>, at: Duration) -> TimerListener {
		let (trigger, timer) = Timer::new();
		timers_by_time
			.entry(at)
			.or_insert_with(VecDeque::new)
			.push_back(trigger);
		timer
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
						*self.current_time.write().expect("RwLock was poisoned") = key;
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

	/// Current test time, starts with 0 once a new TimerRegistry is created and then
	/// increases on every call to [`advance_time`].
	pub fn current_time(&self) -> Duration {
		*self.current_time.read().expect("RwLock was poisoned")
	}
}
