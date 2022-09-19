mod time_handler_guard;
mod timer;
mod timer_registry;

pub use time_handler_guard::TimeHandlerGuard;
pub use timer_registry::TimerRegistry;

#[macro_export]
macro_rules! tokio_test {
	($test:block) => {
		::tokio::runtime::Builder::new_current_thread()
			.build()
			.expect("Failed to build runtime")
			.block_on(async { $test })
	};
}
