#![doc = include_str!("../README.md")]
use std::future::Future;
use std::time::Duration;

mod instant;
use crate::interval::Interval;
pub use instant::Instant;

#[cfg(feature = "mock")]
pub use async_time_mock_core as core;

mod elapsed;
mod interval;
mod timeout;
pub use timeout::Timeout;

#[derive(Clone)]
pub enum MockableClock {
	Real,
	#[cfg(feature = "mock")]
	Mock(std::sync::Arc<async_time_mock_core::TimerRegistry>),
}

pub enum TimeHandlerGuard {
	Real,
	#[cfg(feature = "mock")]
	Mock(async_time_mock_core::TimeHandlerGuard),
}

#[cfg(feature = "mock")]
impl From<async_time_mock_core::TimeHandlerGuard> for TimeHandlerGuard {
	fn from(guard: async_time_mock_core::TimeHandlerGuard) -> Self {
		Self::Mock(guard)
	}
}

impl MockableClock {
	#[cfg(feature = "mock")]
	pub fn mock() -> (Self, std::sync::Arc<async_time_mock_core::TimerRegistry>) {
		let timer_registry = std::sync::Arc::new(async_time_mock_core::TimerRegistry::default());
		(Self::Mock(timer_registry.clone()), timer_registry)
	}

	pub fn now(&self) -> Instant {
		use MockableClock::*;
		match self {
			Real => tokio::time::Instant::now().into(),
			#[cfg(feature = "mock")]
			Mock(registry) => registry.now().into(),
		}
	}

	pub async fn sleep(&self, duration: Duration) -> TimeHandlerGuard {
		use MockableClock::*;
		match self {
			Real => {
				tokio::time::sleep(duration).await;
				TimeHandlerGuard::Real
			}
			#[cfg(feature = "mock")]
			Mock(registry) => registry.sleep(duration).await.into(),
		}
	}

	pub async fn sleep_until(&self, until: Instant) -> TimeHandlerGuard {
		match (self, until) {
			(MockableClock::Real, Instant::Real(until)) => {
				tokio::time::sleep_until(until).await;
				TimeHandlerGuard::Real
			}
			#[cfg(feature = "mock")]
			(MockableClock::Mock(registry), Instant::Mock(until)) => registry.sleep_until(until).await.into(),
			#[cfg(feature = "mock")]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}

	pub fn interval(&self, period: Duration) -> Interval {
		use MockableClock::*;
		match self {
			Real => tokio::time::interval(period).into(),
			#[cfg(feature = "mock")]
			Mock(registry) => registry.interval(period).into(),
		}
	}

	pub fn interval_at(&self, start: Instant, period: Duration) -> Interval {
		match (self, start) {
			(MockableClock::Real, Instant::Real(start)) => tokio::time::interval_at(start, period).into(),
			#[cfg(feature = "mock")]
			(MockableClock::Mock(registry), Instant::Mock(start)) => registry.interval_at(start, period).into(),
			#[cfg(feature = "mock")]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}

	pub fn timeout<T>(&self, duration: Duration, future: T) -> Timeout<T>
	where
		T: Future,
	{
		use MockableClock::*;
		match self {
			Real => tokio::time::timeout(duration, future).into(),
			#[cfg(feature = "mock")]
			Mock(registry) => registry.timeout(duration, future).into(),
		}
	}

	pub fn timeout_at<T>(&self, deadline: Instant, future: T) -> Timeout<T>
	where
		T: Future,
	{
		match (self, deadline) {
			(MockableClock::Real, Instant::Real(deadline)) => tokio::time::timeout_at(deadline, future).into(),
			#[cfg(feature = "mock")]
			(MockableClock::Mock(registry), Instant::Mock(deadline)) => registry.timeout_at(deadline, future).into(),
			#[cfg(feature = "mock")]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}
}
