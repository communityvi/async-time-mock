use crate::{Instant, TimeHandlerGuard};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::MissedTickBehavior;

#[derive(Debug)]
pub enum Interval {
	Real(tokio::time::Interval),
	#[cfg(test)]
	Mock(async_time_mock_core::Interval),
}

impl From<tokio::time::Interval> for Interval {
	fn from(interval: tokio::time::Interval) -> Self {
		Self::Real(interval)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Interval> for Interval {
	fn from(interval: async_time_mock_core::Interval) -> Self {
		Self::Mock(interval)
	}
}

impl Interval {
	pub async fn tick(&mut self) -> (TimeHandlerGuard, Instant) {
		use Interval::*;
		match self {
			Real(interval) => {
				let instant = interval.tick().await;
				(TimeHandlerGuard::Real, instant.into())
			}
			#[cfg(test)]
			Mock(interval) => {
				let (guard, instant) = interval.tick().await;
				(guard.into(), instant.into())
			}
		}
	}

	pub fn poll_tick(&mut self, context: &mut Context<'_>) -> Poll<(TimeHandlerGuard, Instant)> {
		use Interval::*;
		match self {
			Real(interval) => interval
				.poll_tick(context)
				.map(|instant| (TimeHandlerGuard::Real, instant.into())),
			#[cfg(test)]
			Mock(interval) => interval
				.poll_tick(context)
				.map(|(guard, instant)| (guard.into(), instant.into())),
		}
	}

	pub fn reset(&mut self) {
		use Interval::*;
		match self {
			Real(interval) => interval.reset(),
			#[cfg(test)]
			Mock(interval) => interval.reset(),
		}
	}

	pub fn missed_tick_behavior(&self) -> MissedTickBehavior {
		use Interval::*;
		match self {
			Real(interval) => interval.missed_tick_behavior(),
			#[cfg(test)]
			Mock(interval) => match interval.missed_tick_behavior() {
				async_time_mock_core::MissedTickBehavior::Burst => MissedTickBehavior::Burst,
				async_time_mock_core::MissedTickBehavior::Delay => MissedTickBehavior::Delay,
				async_time_mock_core::MissedTickBehavior::Skip => MissedTickBehavior::Skip,
			},
		}
	}

	pub fn set_missed_tick_behavior(&mut self, missed_tick_behavior: MissedTickBehavior) {
		use Interval::*;
		match self {
			Real(interval) => interval.set_missed_tick_behavior(missed_tick_behavior),
			#[cfg(test)]
			Mock(interval) => interval.set_missed_tick_behavior(match missed_tick_behavior {
				MissedTickBehavior::Burst => async_time_mock_core::MissedTickBehavior::Burst,
				MissedTickBehavior::Delay => async_time_mock_core::MissedTickBehavior::Delay,
				MissedTickBehavior::Skip => async_time_mock_core::MissedTickBehavior::Skip,
			}),
		}
	}

	pub fn period(&self) -> Duration {
		use Interval::*;
		match self {
			Real(interval) => interval.period(),
			#[cfg(test)]
			Mock(interval) => interval.period(),
		}
	}
}

#[cfg(feature = "stream")]
impl futures_core::stream::Stream for Interval {
	type Item = (TimeHandlerGuard, Instant);

	fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
		use Interval::*;
		match this {
			Real(interval) => interval
				.poll_tick(context)
				.map(|instant| Some((TimeHandlerGuard::Real, instant.into()))),
			#[cfg(test)]
			Mock(interval) => interval
				.poll_tick(context)
				.map(|(guard, instant)| Some((guard.into(), instant.into()))),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(usize::MAX, None)
	}
}