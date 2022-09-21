use async_io::Timer;
use std::time::Duration;

mod instant;
pub use instant::Instant;

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
				Timer::after(duration).await;
				TimeHandlerGuard::Real
			}
			#[cfg(test)]
			Mock(registry) => registry.sleep(duration).await.into(),
		}
	}
}
