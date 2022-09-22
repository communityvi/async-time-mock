use super::TimeHandlerGuard;
use async_std::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub enum Interval {
	Real(async_std::stream::Interval),
	#[cfg(test)]
	Mock(async_time_mock_core::Interval),
}

impl From<async_std::stream::Interval> for Interval {
	fn from(interval: async_std::stream::Interval) -> Self {
		Self::Real(interval)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Interval> for Interval {
	fn from(interval: async_time_mock_core::Interval) -> Self {
		Self::Mock(interval)
	}
}

impl Stream for Interval {
	type Item = TimeHandlerGuard;

	fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
		use Interval::*;
		match this {
			Real(interval) => Pin::new(interval)
				.poll_next(context)
				.map(|option| option.map(|_| TimeHandlerGuard::Real)),
			#[cfg(test)]
			Mock(interval) => interval.poll_tick(context).map(|(guard, _)| Some(guard.into())),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(usize::MAX, None)
	}
}
