use crate::TimerRegistry;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Instant {
	duration: Duration,
	timer_registry_id: u64,
}

impl Instant {
	pub(crate) const fn new(duration: Duration, timer_registry_id: u64) -> Self {
		Self {
			duration,
			timer_registry_id,
		}
	}

	pub(crate) const fn into_duration(self, timer_registry_id: u64) -> Duration {
		if self.timer_registry_id != timer_registry_id {
			panic!("Can't use Instants from one TimerRegistry in another TimerRegistry.");
		}
		self.duration
	}

	// std::time::Instant::now() isn't supported because it would require a TimerRegistry

	/// Equivalent to [`std::time::Instant::duration_since`].
	pub const fn duration_since(&self, earlier: Self) -> Duration {
		self.assert_instances_are_compatible(&earlier);
		self.duration.saturating_sub(earlier.duration)
	}

	/// Equivalent to [`std::time::Instant::checked_duration_since`].
	pub const fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
		self.assert_instances_are_compatible(&earlier);
		self.duration.checked_sub(earlier.duration)
	}

	/// Equivalent to [`std::time::Instant::saturated_duration_since`].
	pub const fn saturated_duration_since(&self, earlier: Self) -> Duration {
		self.assert_instances_are_compatible(&earlier);
		self.duration.saturating_sub(earlier.duration)
	}

	/// Similar to [`std::time::Instant::elapsed`], but needs a [`TimerRegistry`] to calculate the time that has passed.
	///
	/// # Panics
	/// If the [`TimerRegistry`] passed in was different than the one this `Instant` was created with.
	pub fn elapsed(&self, timer_registry: &TimerRegistry) -> Duration {
		timer_registry.now().duration_since(*self)
	}

	/// Equivalent to [`std::time::Instant::checked_add`].
	pub const fn checked_add(&self, duration: Duration) -> Option<Self> {
		let timer_registry_id = self.timer_registry_id;
		match self.duration.checked_add(duration) {
			Some(duration) => Some(Self {
				duration,
				timer_registry_id,
			}),
			None => None,
		}
	}

	/// Equivalent to [`std::time::Instant::checked_sub`].
	pub const fn checked_sub(&self, duration: Duration) -> Option<Self> {
		let timer_registry_id = self.timer_registry_id;
		match self.duration.checked_sub(duration) {
			Some(duration) => Some(Self {
				duration,
				timer_registry_id,
			}),
			None => None,
		}
	}

	const fn assert_instances_are_compatible(&self, other: &Self) {
		if self.timer_registry_id != other.timer_registry_id {
			panic!("Operations between Instant's from different TimerRegistry instances are not supported.");
		}
	}
}

impl Add<Duration> for Instant {
	type Output = Instant;

	fn add(self, rhs: Duration) -> Self::Output {
		Self {
			duration: self.duration.add(rhs),
			timer_registry_id: self.timer_registry_id,
		}
	}
}

impl AddAssign<Duration> for Instant {
	fn add_assign(&mut self, rhs: Duration) {
		self.duration.add_assign(rhs);
	}
}

impl Sub<Duration> for Instant {
	type Output = Instant;

	fn sub(self, rhs: Duration) -> Self::Output {
		Self {
			duration: self.duration.sub(rhs),
			timer_registry_id: self.timer_registry_id,
		}
	}
}

impl Sub<Instant> for Instant {
	type Output = Duration;

	fn sub(self, rhs: Instant) -> Self::Output {
		self.assert_instances_are_compatible(&rhs);
		self.duration.sub(rhs.duration)
	}
}

impl SubAssign<Duration> for Instant {
	fn sub_assign(&mut self, rhs: Duration) {
		self.duration.sub_assign(rhs);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::time::Duration;

	#[test]
	#[should_panic]
	fn should_not_allow_fetching_duration_from_incorrect_timer_registry() {
		Instant::new(Duration::ZERO, 0).into_duration(1);
	}
}
