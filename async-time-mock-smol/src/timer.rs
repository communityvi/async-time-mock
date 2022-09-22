use crate::{Instant, TimeHandlerGuard};
use futures_core::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub enum Timer {
	Real(async_io::Timer),
	#[cfg(test)]
	MockInterval(async_time_mock_core::Interval),
	// TODO: sleep, if we ever want to support the Timer methods below
}

impl From<async_io::Timer> for Timer {
	fn from(timer: async_io::Timer) -> Self {
		Self::Real(timer)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Interval> for Timer {
	fn from(interval: async_time_mock_core::Interval) -> Self {
		Self::MockInterval(interval)
	}
}

impl Timer {
	// Timer::never can't determine if it should real or mock, therefore omitted

	// Timer::after isn't supported because it would require a TimerRegistry
	// Timer::at isn't supported because it would require a TimerRegistry
	// Timer::interval isn't supported because it would require a TimerRegistry
	// Timer::interval_at isn't supported because it would require a TimerRegistry
	// Timer::set_after isn't implemented and is unsure if will be
	// Timer::set_at isn't implemented and is unsure if will be
	// Timer::set_interval isn't implemented and is unsure if will be
	// Timer::set_interval_at isn't implemented and is unsure if will be
}

impl Future for Timer {
	type Output = (TimeHandlerGuard, Instant);

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		use Timer::*;
		match this {
			Real(timer) => Pin::new(timer)
				.poll(context)
				.map(|instant| (TimeHandlerGuard::Real, instant.into())),
			#[cfg(test)]
			MockInterval(interval) => interval
				.poll_tick(context)
				.map(|(guard, instant)| (guard.into(), instant.into())),
		}
	}
}

impl Stream for Timer {
	type Item = (TimeHandlerGuard, Instant);

	fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
		use Timer::*;
		match this {
			Real(timer) => Pin::new(timer)
				.poll_next(context)
				.map(|option| option.map(|instant| (TimeHandlerGuard::Real, instant.into()))),
			#[cfg(test)]
			MockInterval(interval) => interval
				.poll_tick(context)
				.map(|(guard, instant)| Some((guard.into(), instant.into()))),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(usize::MAX, None)
	}
}
