use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Instant {
	Real(tokio::time::Instant),
	#[cfg(feature = "mock")]
	Mock(async_time_mock_core::Instant),
}

impl From<tokio::time::Instant> for Instant {
	fn from(instant: tokio::time::Instant) -> Self {
		Self::Real(instant)
	}
}

#[cfg(feature = "mock")]
impl From<async_time_mock_core::Instant> for Instant {
	fn from(instant: async_time_mock_core::Instant) -> Self {
		Self::Mock(instant)
	}
}

impl Instant {
	// std::time::Instant::now() isn't supported because it would require a TimerRegistry

	/// Equivalent to [`std::time::Instant::duration_since`].
	///
	/// # Panics
	/// If `self` and `earlier` aren't either both mock or both real.
	pub fn duration_since(&self, earlier: Self) -> Duration {
		match (self, earlier) {
			(Instant::Real(this), Instant::Real(earlier)) => this.duration_since(earlier),
			#[cfg(feature = "mock")]
			(Instant::Mock(this), Instant::Mock(earlier)) => this.duration_since(earlier),
			#[cfg(feature = "mock")]
			_ => panic!("Instants weren't compatible, both need to be either real or mocked"),
		}
	}

	/// Equivalent to [`std::time::Instant::checked_duration_since`].
	pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
		match (self, earlier) {
			(Instant::Real(this), Instant::Real(earlier)) => this.checked_duration_since(earlier),
			#[cfg(feature = "mock")]
			(Instant::Mock(this), Instant::Mock(earlier)) => this.checked_duration_since(earlier),
			#[cfg(feature = "mock")]
			_ => panic!("Instants weren't compatible, both need to be either real or mocked"),
		}
	}

	/// Equivalent to [`std::time::Instant::saturated_duration_since`].
	pub fn saturated_duration_since(&self, earlier: Self) -> Duration {
		match (self, earlier) {
			(Instant::Real(this), Instant::Real(earlier)) => this.saturating_duration_since(earlier),
			#[cfg(feature = "mock")]
			(Instant::Mock(this), Instant::Mock(earlier)) => this.saturated_duration_since(earlier),
			#[cfg(feature = "mock")]
			_ => panic!("Instants weren't compatible, both need to be either real or mocked"),
		}
	}

	// std::time::Instant::elapsed() isn't supported because it would require a TimerRegistry

	/// Equivalent to [`std::time::Instant::checked_add`].
	pub fn checked_add(&self, duration: Duration) -> Option<Self> {
		use Instant::*;
		match self {
			Real(this) => this.checked_add(duration).map(Into::into),
			#[cfg(feature = "mock")]
			Mock(this) => this.checked_add(duration).map(Into::into),
		}
	}

	/// Equivalent to [`std::time::Instant::checked_sub`].
	pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
		use Instant::*;
		match self {
			Real(this) => this.checked_sub(duration).map(Into::into),
			#[cfg(feature = "mock")]
			Mock(this) => this.checked_sub(duration).map(Into::into),
		}
	}
}

impl PartialOrd for Instant {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Instant::Real(this), Instant::Real(other)) => this.partial_cmp(other),
			#[cfg(feature = "mock")]
			(Instant::Mock(this), Instant::Mock(other)) => this.partial_cmp(other),
			#[cfg(feature = "mock")]
			_ => panic!("Instants weren't compatible, both need to be either real or mocked"),
		}
	}
}

impl Ord for Instant {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Instant::Real(this), Instant::Real(other)) => this.cmp(other),
			#[cfg(feature = "mock")]
			(Instant::Mock(this), Instant::Mock(other)) => this.cmp(other),
			#[cfg(feature = "mock")]
			_ => panic!("Instants weren't compatible, both need to be either real or mocked"),
		}
	}
}

impl Add<Duration> for Instant {
	type Output = Instant;

	fn add(self, rhs: Duration) -> Self::Output {
		use Instant::*;
		match self {
			Real(instant) => instant.add(rhs).into(),
			#[cfg(feature = "mock")]
			Mock(instant) => instant.add(rhs).into(),
		}
	}
}

impl AddAssign<Duration> for Instant {
	fn add_assign(&mut self, rhs: Duration) {
		use Instant::*;
		match self {
			Real(instant) => instant.add_assign(rhs),
			#[cfg(feature = "mock")]
			Mock(instant) => instant.add_assign(rhs),
		}
	}
}

impl Sub<Duration> for Instant {
	type Output = Instant;

	fn sub(self, rhs: Duration) -> Self::Output {
		use Instant::*;
		match self {
			Real(instant) => instant.sub(rhs).into(),
			#[cfg(feature = "mock")]
			Mock(instant) => instant.sub(rhs).into(),
		}
	}
}

impl SubAssign<Duration> for Instant {
	fn sub_assign(&mut self, rhs: Duration) {
		use Instant::*;
		match self {
			Real(instant) => instant.sub_assign(rhs),
			#[cfg(feature = "mock")]
			Mock(instant) => instant.sub_assign(rhs),
		}
	}
}
