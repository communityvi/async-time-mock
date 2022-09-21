use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Instant(Duration);

impl Instant {
	pub(crate) fn new(duration: Duration) -> Self {
		Self(duration)
	}

	pub(crate) fn into_duration(self) -> Duration {
		self.0
	}

	// std::time::Instant::now() isn't supported because it would require a TimerRegistry

	/// Equivalent to [`std::time::Instant::duration_since`].
	pub fn duration_since(&self, earlier: Self) -> Duration {
		self.0 - earlier.0
	}

	/// Equivalent to [`std::time::Instant::checked_duration_since`].
	pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
		self.0.checked_sub(earlier.0)
	}

	/// Equivalent to [`std::time::Instant::saturated_duration_since`].
	pub fn saturated_duration_since(&self, earlier: Self) -> Duration {
		self.0.saturating_sub(earlier.0)
	}

	// std::time::Instant::elapsed() isn't supported because it would require a TimerRegistry

	/// Equivalent to [`std::time::Instant::checked_add`].
	pub fn checked_add(&self, duration: Duration) -> Option<Self> {
		self.0.checked_add(duration).map(Self)
	}

	/// Equivalent to [`std::time::Instant::checked_sub`].
	pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
		self.0.checked_sub(duration).map(Self)
	}
}

impl Add<Duration> for Instant {
	type Output = Instant;

	fn add(self, rhs: Duration) -> Self::Output {
		Self(self.0.add(rhs))
	}
}

impl AddAssign<Duration> for Instant {
	fn add_assign(&mut self, rhs: Duration) {
		self.0.add_assign(rhs);
	}
}

impl Sub<Duration> for Instant {
	type Output = Instant;

	fn sub(self, rhs: Duration) -> Self::Output {
		Self(self.0.sub(rhs))
	}
}

impl Sub<Instant> for Instant {
	type Output = Duration;

	fn sub(self, rhs: Instant) -> Self::Output {
		self.0.sub(rhs.0)
	}
}

impl SubAssign<Duration> for Instant {
	fn sub_assign(&mut self, rhs: Duration) {
		self.0.sub_assign(rhs);
	}
}
