use std::time::Duration;

mod instant;
pub use instant::Instant;

mod timer;
pub use timer::Timer;

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
				async_io::Timer::after(duration).await;
				TimeHandlerGuard::Real
			}
			#[cfg(test)]
			Mock(registry) => registry.sleep(duration).await.into(),
		}
	}

	pub async fn sleep_until(&self, until: Instant) -> TimeHandlerGuard {
		match (self, until) {
			(MockableClock::Real, Instant::Real(until)) => {
				async_io::Timer::at(until).await;
				TimeHandlerGuard::Real
			}
			#[cfg(test)]
			(MockableClock::Mock(registry), Instant::Mock(until)) => registry.sleep_until(until).await.into(),
			#[cfg(test)]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}

	pub async fn interval(&self, period: Duration) -> Timer {
		use MockableClock::*;
		match self {
			Real => async_io::Timer::interval(period).into(),
			#[cfg(test)]
			Mock(registry) => registry.interval(period).into(),
		}
	}

	pub async fn interval_at(&self, start: Instant, period: Duration) -> Timer {
		match (self, start) {
			(MockableClock::Real, Instant::Real(start)) => async_io::Timer::interval_at(start, period).into(),
			#[cfg(test)]
			(MockableClock::Mock(registry), Instant::Mock(start)) => registry.interval_at(start, period).into(),
			#[cfg(test)]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}
}
