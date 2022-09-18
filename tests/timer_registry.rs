use async_time_mock::TimerRegistry;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn sleep_should_finish_if_time_is_advanced_by_exactly_sleep_amount() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let has_slept = Arc::new(AtomicBool::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_slept = has_slept.clone();
		async move {
			let _guard = timer_registry.sleep(Duration::from_secs(10)).await;
			has_slept.store(true, Ordering::SeqCst);
			assert_eq!(Duration::from_secs(10), timer_registry.current_time());
		}
	});

	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should not have slept before the time was advanced"
	);
	timer_registry.advance_time(Duration::from_secs(10)).await;
	assert!(
		has_slept.load(Ordering::SeqCst),
		"Should have slept when advancing time"
	);
	join_handle.await.expect("Sleeping task crashed");
}
