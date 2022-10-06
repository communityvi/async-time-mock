use crate::elapsed::Elapsed;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub enum Timeout<T> {
	Real(tokio::time::Timeout<T>),
	#[cfg(test)]
	Mock(async_time_mock_core::Timeout<T>),
}

impl<T> Timeout<T> {
	pub fn get_ref(&self) -> &T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.get_ref(),
			#[cfg(test)]
			Mock(timeout) => timeout.get_ref(),
		}
	}

	pub fn get_mut(&mut self) -> &mut T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.get_mut(),
			#[cfg(test)]
			Mock(timeout) => timeout.get_mut(),
		}
	}

	pub fn into_inner(self) -> T {
		use Timeout::*;
		match self {
			Real(timeout) => timeout.into_inner(),
			#[cfg(test)]
			Mock(timeout) => timeout.into_inner(),
		}
	}
}

impl<T: Debug> Debug for Timeout<T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		use Timeout::*;
		match self {
			Real(timeout) => Debug::fmt(timeout, formatter),
			#[cfg(test)]
			Mock(_) => formatter.debug_struct("Mock(Timeout)").finish(),
		}
	}
}

// implementing `Unpin` is currently impossible because the underlying async_time_mock_core::Timeout doesn't allow it (`sleep` is an async function)

impl<T> Future for Timeout<T>
where
	T: Future,
{
	type Output = Result<T::Output, Elapsed>;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		// SAFETY: `this` is never used to move the underlying data.
		let this = unsafe { self.get_unchecked_mut() };
		use Timeout::*;
		match this {
			Real(timeout) => {
				// SAFETY: `this` comes from a `self: Pin<&mut Self>` therefore `timeout` is already
				// transitively pinned.
				unsafe { Pin::new_unchecked(timeout) }
					.poll(context)
					.map(|result| result.map_err(Into::into))
			}
			#[cfg(test)]
			Mock(timeout) => {
				// SAFETY: `this` comes from a `self: Pin<&mut Self>` therefore `timeout` is already
				// transitively pinned.
				unsafe { Pin::new_unchecked(timeout) }
					.poll(context)
					.map(|result| result.map_err(Into::into))
			}
		}
	}
}

impl<T> From<tokio::time::Timeout<T>> for Timeout<T> {
	fn from(timeout: tokio::time::Timeout<T>) -> Self {
		Self::Real(timeout)
	}
}

#[cfg(test)]
impl<T> From<async_time_mock_core::Timeout<T>> for Timeout<T> {
	fn from(timeout: async_time_mock_core::Timeout<T>) -> Self {
		Self::Mock(timeout)
	}
}
