use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// NOTE That the can't implement PartialEq, Eq, Clone or Copy, because TimeHandlerGuard doesn't support that.
#[must_use = "Elapsed must only be dropped once all side-effects of the timeout have been handled."]
#[derive(Debug)]
pub enum Elapsed {
	Real(tokio::time::error::Elapsed),
	#[cfg(test)]
	Mock(async_time_mock_core::Elapsed),
}

impl Display for Elapsed {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		use Elapsed::*;
		match self {
			Real(error) => Display::fmt(error, formatter),
			#[cfg(test)]
			Mock(elapsed) => Display::fmt(elapsed, formatter),
		}
	}
}

impl Error for Elapsed {}

impl From<tokio::time::error::Elapsed> for Elapsed {
	fn from(error: tokio::time::error::Elapsed) -> Self {
		Self::Real(error)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Elapsed> for Elapsed {
	fn from(elapsed: async_time_mock_core::Elapsed) -> Self {
		Self::Mock(elapsed)
	}
}
