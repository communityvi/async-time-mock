use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// NOTE That the can't implement PartialEq, Eq, Clone or Copy, because TimeHandlerGuard doesn't support that.
#[must_use = "TimeoutError must only be dropped once all side-effects of the timeout have been handled."]
#[derive(Debug)]
pub enum TimeoutError {
	Real(async_std::future::TimeoutError),
	#[cfg(test)]
	Mock(async_time_mock_core::Elapsed),
}

impl Display for TimeoutError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		use TimeoutError::*;
		match self {
			Real(error) => Display::fmt(error, formatter),
			#[cfg(test)]
			Mock(elapsed) => Display::fmt(elapsed, formatter),
		}
	}
}

impl Error for TimeoutError {}

impl From<async_std::future::TimeoutError> for TimeoutError {
	fn from(error: async_std::future::TimeoutError) -> Self {
		Self::Real(error)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Elapsed> for TimeoutError {
	fn from(elapsed: async_time_mock_core::Elapsed) -> Self {
		Self::Mock(elapsed)
	}
}
