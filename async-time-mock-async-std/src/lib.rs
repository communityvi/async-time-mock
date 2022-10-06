use async_std::task::sleep;
use std::future::Future;
use std::time::Duration;

mod instant;
pub use instant::Instant;
mod timeout_error;
pub use timeout_error::TimeoutError;

pub use async_time_mock_core;

#[cfg(feature = "unstable")]
mod interval;
#[cfg(feature = "unstable")]
pub use interval::Interval;

#[derive(Clone)]
pub enum MockableClock {
	Real,
	#[cfg(test)]
	Mock(std::sync::Arc<async_time_mock_core::TimerRegistry>),
}

pub enum TimeHandlerGuard {
	Real,
	#[cfg(test)]
	Mock(async_time_mock_core::TimeHandlerGuard),
}

#[cfg(test)]
impl From<async_time_mock_core::TimeHandlerGuard> for TimeHandlerGuard {
	fn from(guard: async_time_mock_core::TimeHandlerGuard) -> Self {
		Self::Mock(guard)
	}
}

impl MockableClock {
	#[cfg(test)]
	pub fn mock() -> (Self, std::sync::Arc<async_time_mock_core::TimerRegistry>) {
		let timer_registry = std::sync::Arc::new(async_time_mock_core::TimerRegistry::default());
		(Self::Mock(timer_registry.clone()), timer_registry)
	}

	pub fn now(&self) -> Instant {
		use MockableClock::*;
		match self {
			Real => std::time::Instant::now().into(),
			#[cfg(test)]
			Mock(registry) => registry.now().into(),
		}
	}

	pub async fn sleep(&self, duration: Duration) -> TimeHandlerGuard {
		use MockableClock::*;
		match self {
			Real => {
				sleep(duration).await;
				TimeHandlerGuard::Real
			}
			#[cfg(test)]
			Mock(registry) => registry.sleep(duration).await.into(),
		}
	}

	// AFAIK, async-std doesn't have any functionality equivalent to sleep_until

	#[cfg(feature = "unstable")]
	pub fn interval(&self, period: Duration) -> Interval {
		use MockableClock::*;
		match self {
			Real => async_std::stream::interval(period).into(),
			#[cfg(test)]
			Mock(registry) => registry.interval(period).into(),
		}
	}

	pub async fn timeout<F, T>(&self, duration: Duration, future: F) -> Result<T, TimeoutError>
	where
		F: Future<Output = T>,
	{
		use MockableClock::*;
		match self {
			Real => async_std::future::timeout(duration, future).await.map_err(Into::into),
			#[cfg(test)]
			Mock(registry) => registry.timeout(duration, future).await.map_err(Into::into),
		}
	}

	// AFAIK, async-std doesn't have any functionality equivalent to timeout_at
}
