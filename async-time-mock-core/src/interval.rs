use crate::{Instant, TimeHandlerGuard, TimerRegistry};
use std::fmt::{Debug, Formatter};
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

pub struct Interval {
	timer_registry: Arc<TimerRegistry>,
	sleep: Pin<Box<dyn Future<Output = TimeHandlerGuard> + Send>>,
	next_deadline: Instant,
	period: Duration,
	missed_tick_behavior: MissedTickBehavior,
	missed_tick_threshold: Duration,
}

impl Interval {
	pub(crate) fn new(timer_registry: Arc<TimerRegistry>, start: Instant, period: Duration) -> Self {
		let sleep = Box::pin(timer_registry.sleep_until(start));
		Self {
			timer_registry,
			sleep,
			next_deadline: start,
			period,
			missed_tick_behavior: MissedTickBehavior::Burst,
			missed_tick_threshold: Duration::ZERO,
		}
	}

	pub async fn tick(&mut self) -> (TimeHandlerGuard, Instant) {
		poll_fn(|context| self.poll_tick(context)).await
	}

	pub fn poll_tick(&mut self, context: &mut Context<'_>) -> Poll<(TimeHandlerGuard, Instant)> {
		use Poll::*;
		let guard = match self.sleep.as_mut().poll(context) {
			Ready(guard) => guard,
			Pending => return Pending,
		};

		let now = self.timer_registry.now();
		let tick_time = self.next_deadline;

		if now > (tick_time + self.missed_tick_threshold) {
			use MissedTickBehavior::*;
			self.next_deadline = match self.missed_tick_behavior {
				Burst => tick_time + self.period,
				Delay => now + self.period,
				Skip => {
					let mut new_deadline = tick_time + self.period;
					while new_deadline < now {
						new_deadline += self.period;
					}
					new_deadline
				}
			};
		} else {
			self.next_deadline = tick_time + self.period;
		}

		self.sleep = Box::pin(self.timer_registry.sleep_until(self.next_deadline));

		Ready((guard, tick_time))
	}

	pub fn reset(&mut self) {
		let now = self.timer_registry.now();
		self.next_deadline = now + self.period;
		self.sleep = Box::pin(self.timer_registry.sleep_until(self.next_deadline));
	}

	pub fn missed_tick_behavior(&self) -> MissedTickBehavior {
		self.missed_tick_behavior
	}

	pub fn set_missed_tick_behavior(&mut self, missed_tick_behavior: MissedTickBehavior) {
		self.missed_tick_behavior = missed_tick_behavior;
	}

	pub fn missed_tick_threshold(&self) -> Duration {
		self.missed_tick_threshold
	}

	pub fn set_missed_tick_threshold(&mut self, missed_tick_threshold: Duration) {
		self.missed_tick_threshold = missed_tick_threshold;
	}

	pub fn period(&self) -> Duration {
		self.period
	}
}

impl Debug for Interval {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let Self {
			timer_registry,
			sleep: _,
			next_deadline,
			period,
			missed_tick_behavior,
			missed_tick_threshold,
		} = self;
		formatter
			.debug_struct("Interval")
			.field("timer_registry", timer_registry)
			.field("sleep", &"impl Future<Output = TimeHandlerGuard>")
			.field("next_deadline", next_deadline)
			.field("period", period)
			.field("missed_tick_behavior", missed_tick_behavior)
			.field("missed_tick_threshold", missed_tick_threshold)
			.finish()
	}
}

/// Same as [`tokio::time::MissedTickBehavior`]
#[derive(Debug, Copy, Clone)]
pub enum MissedTickBehavior {
	Burst,
	Delay,
	Skip,
}
