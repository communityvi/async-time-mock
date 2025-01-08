use crate::elapsed::Elapsed;
use pin_project::pin_project;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project(project = ProjectedTimeout)]
pub enum Timeout<T> {
	Real(#[pin] tokio::time::Timeout<T>),
	#[cfg(feature = "mock")]
	Mock(#[pin] async_time_mock_core::Timeout<T>),
}

impl<T> Timeout<T> {
	pub fn get_ref(&self) -> &T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.get_ref(),
			#[cfg(feature = "mock")]
			Mock(timeout) => timeout.get_ref(),
		}
	}

	pub fn get_mut(&mut self) -> &mut T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.get_mut(),
			#[cfg(feature = "mock")]
			Mock(timeout) => timeout.get_mut(),
		}
	}

	pub fn into_inner(self) -> T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.into_inner(),
			#[cfg(feature = "mock")]
			Mock(timeout) => timeout.into_inner(),
		}
	}
}

impl<T: Debug> Debug for Timeout<T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		use Timeout::*;
		match self {
			Real(timeout) => Debug::fmt(timeout, formatter),
			#[cfg(feature = "mock")]
			Mock(_) => formatter.debug_struct("Mock(Timeout)").finish(),
		}
	}
}

impl<T> Future for Timeout<T>
where
	T: Future,
{
	type Output = Result<T::Output, Elapsed>;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		use ProjectedTimeout::*;
		match self.project() {
			Real(timeout) => timeout.poll(context).map(|result| result.map_err(Into::into)),
			#[cfg(feature = "mock")]
			Mock(timeout) => timeout.poll(context).map(|result| result.map_err(Into::into)),
		}
	}
}

impl<T> From<tokio::time::Timeout<T>> for Timeout<T> {
	fn from(timeout: tokio::time::Timeout<T>) -> Self {
		Self::Real(timeout)
	}
}

#[cfg(feature = "mock")]
impl<T> From<async_time_mock_core::Timeout<T>> for Timeout<T> {
	fn from(timeout: async_time_mock_core::Timeout<T>) -> Self {
		Self::Mock(timeout)
	}
}
