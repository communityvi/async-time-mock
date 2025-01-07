use crate::TimeHandlerGuard;
#[cfg(feature = "mock")]
use async_time_mock_core::TimerListener;
use pin_project::pin_project;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

#[pin_project(project = SleepProjection)]
#[derive(Debug)]
pub enum Sleep {
	Real(#[pin] tokio::time::Sleep),
	#[cfg(feature = "mock")]
	Mock(#[pin] TimerListener),
}

impl From<tokio::time::Sleep> for Sleep {
	fn from(sleep: tokio::time::Sleep) -> Self {
		Self::Real(sleep)
	}
}
#[cfg(feature = "mock")]
impl From<TimerListener> for Sleep {
	fn from(listener: TimerListener) -> Self {
		Self::Mock(listener)
	}
}
impl Future for Sleep {
	type Output = TimeHandlerGuard;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.project();

		use SleepProjection::*;
		match this {
			Real(sleep) => {
				ready!(sleep.poll(context));
				Poll::Ready(TimeHandlerGuard::Real)
			}
			#[cfg(feature = "mock")]
			Mock(listener) => {
				let guard = ready!(listener.poll(context));
				Poll::Ready(guard.into())
			}
		}
	}
}
