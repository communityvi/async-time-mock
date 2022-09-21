pub enum Instant {
	Real(std::time::Instant),
	#[cfg(test)]
	Mock(async_time_mock_core::Instant),
}

impl From<std::time::Instant> for Instant {
	fn from(instant: std::time::Instant) -> Self {
		Self::Real(instant)
	}
}

#[cfg(test)]
impl From<async_time_mock_core::Instant> for Instant {
	fn from(instant: async_time_mock_core::Instant) -> Self {
		Self::Mock(instant)
	}
}
