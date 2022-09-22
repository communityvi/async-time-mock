use std::time::Duration;

mod instant;
use crate::interval::Interval;
pub use instant::Instant;

mod interval;

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
			Real => tokio::time::Instant::now().into(),
			#[cfg(test)]
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
			#[cfg(test)]
			Mock(registry) => registry.sleep(duration).await.into(),
		}
	}

	pub async fn sleep_until(&self, until: Instant) -> TimeHandlerGuard {
		match (self, until) {
			(MockableClock::Real, Instant::Real(until)) => {
				tokio::time::sleep_until(until).await;
				TimeHandlerGuard::Real
			}
			#[cfg(test)]
			(MockableClock::Mock(registry), Instant::Mock(until)) => registry.sleep_until(until).await.into(),
			#[cfg(test)]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}

	pub fn interval(&self, period: Duration) -> Interval {
		use MockableClock::*;
		match self {
			Real => tokio::time::interval(period).into(),
			#[cfg(test)]
			Mock(registry) => {
				let mut interval = registry.interval(period);
				interval.set_missed_tick_threshold(TOKIO_MISSED_TICK_THRESHOLD);
				interval.into()
			}
		}
	}

	pub fn interval_at(&self, start: Instant, period: Duration) -> Interval {
		match (self, start) {
			(MockableClock::Real, Instant::Real(start)) => tokio::time::interval_at(start, period).into(),
			#[cfg(test)]
			(MockableClock::Mock(registry), Instant::Mock(start)) => {
				let mut interval = registry.interval_at(start, period);
				interval.set_missed_tick_threshold(TOKIO_MISSED_TICK_THRESHOLD);
				interval.into()
			}
			#[cfg(test)]
			_ => panic!("Clock and instant weren't compatible, both need to be either real or mocked"),
		}
	}
}

#[cfg(test)]
// See https://github.com/tokio-rs/tokio/blob/dea1cd49955ab5e9d041e9f1ed0c5f28e18246de/tokio/src/time/interval.rs#L478
const TOKIO_MISSED_TICK_THRESHOLD: Duration = Duration::from_millis(5);
