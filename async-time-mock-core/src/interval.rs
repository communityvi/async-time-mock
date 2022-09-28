use crate::{Instant, TimeHandlerGuard, TimerRegistry};
use std::fmt::{Debug, Formatter};
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use std::time::Duration;

pub struct Interval {
	timer_registry: Arc<TimerRegistry>,
	sleep: Pin<Box<dyn Future<Output = TimeHandlerGuard> + Send>>,
	next_deadline: Instant,
	period: Duration,
}

impl Interval {
	pub(crate) fn new(timer_registry: Arc<TimerRegistry>, start: Instant, period: Duration) -> Self {
		let sleep = Box::pin(timer_registry.sleep_until(start));
		Self {
			timer_registry,
			sleep,
			next_deadline: start,
			period,
		}
	}

	pub async fn tick(&mut self) -> (TimeHandlerGuard, Instant) {
		poll_fn(|context| self.poll_tick(context)).await
	}

	pub fn poll_tick(&mut self, context: &mut Context<'_>) -> Poll<(TimeHandlerGuard, Instant)> {
		let guard = ready!(self.sleep.as_mut().poll(context));

		let tick_time = self.next_deadline;

		self.next_deadline = tick_time + self.period;

		self.sleep = Box::pin(self.timer_registry.sleep_until(self.next_deadline));

		Poll::Ready((guard, tick_time))
	}

	pub fn reset(&mut self) {
		let now = self.timer_registry.now();
		self.next_deadline = now + self.period;
		self.sleep = Box::pin(self.timer_registry.sleep_until(self.next_deadline));
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
		} = self;
		formatter
			.debug_struct("Interval")
			.field("timer_registry", timer_registry)
			.field("sleep", &"impl Future<Output = TimeHandlerGuard>")
			.field("next_deadline", next_deadline)
			.field("period", period)
			.finish()
	}
}
