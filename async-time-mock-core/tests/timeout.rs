use async_time_mock_core::TimerRegistry;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn timeout_should_finish_immediately() {
	let timer_registry = TimerRegistry::default();

	let result = timer_registry
		.timeout(Duration::from_secs(10), std::future::ready(()))
		.await;
	assert!(result.is_ok(), "Timer should have finished immediately.");
}

#[tokio::test]
async fn timeout_should_time_out() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let has_timed_out = Arc::new(AtomicBool::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_timed_out = has_timed_out.clone();
		async move {
			match timer_registry
				.timeout(Duration::from_secs(10), std::future::pending())
				.await
			{
				Ok(()) => panic!("Pending future should not have succeeded"),
				Err(_elapsed) => {
					has_timed_out.store(true, Ordering::SeqCst);
					assert_eq!(Duration::from_secs(10), timer_registry.now() - start);
				}
			}
		}
	});

	assert!(
		!has_timed_out.load(Ordering::SeqCst),
		"Should not have timed out before the time was advanced"
	);
	timer_registry.advance_time(Duration::from_secs(10)).await;
	assert!(
		has_timed_out.load(Ordering::SeqCst),
		"Should have timed out after advancing time"
	);

	join_handle.await.expect("Timeout task crashed");
}

#[tokio::test]
async fn timeout_at_should_finish_immediately() {
	let timer_registry = TimerRegistry::default();
	let now = timer_registry.now();

	let result = timer_registry
		.timeout_at(now + Duration::from_secs(10), std::future::ready(()))
		.await;
	assert!(result.is_ok(), "Timer should have finished immediately.");
}

#[tokio::test]
async fn timeout_at_should_time_out() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let has_timed_out = Arc::new(AtomicBool::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_timed_out = has_timed_out.clone();
		async move {
			match timer_registry
				.timeout_at(start + Duration::from_secs(10), std::future::pending())
				.await
			{
				Ok(()) => panic!("Pending future should not have succeeded"),
				Err(_elapsed) => {
					has_timed_out.store(true, Ordering::SeqCst);
					assert_eq!(Duration::from_secs(10), timer_registry.now() - start);
				}
			}
		}
	});

	assert!(
		!has_timed_out.load(Ordering::SeqCst),
		"Should not have timed out before the time was advanced"
	);
	timer_registry.advance_time(Duration::from_secs(10)).await;
	assert!(
		has_timed_out.load(Ordering::SeqCst),
		"Should have timed out after advancing time"
	);

	join_handle.await.expect("Timeout task crashed");
}
