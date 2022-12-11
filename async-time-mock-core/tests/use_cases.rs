use async_time_mock_core::TimerRegistry;
use std::future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn operation_with_timeout_triggered_by_interval() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let operation_count = Arc::new(AtomicUsize::default());

	let start = timer_registry.now();

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let operation_count = operation_count.clone();
		async move {
			let mut interval = timer_registry.interval(Duration::from_secs(1));
			for seconds in 0..10 {
				let guard = interval.tick().await;
				assert_eq!(Duration::from_secs(seconds), timer_registry.now() - start);

				let timeout = timer_registry.timeout(Duration::from_millis(500), future::pending::<()>());
				drop(guard); // dropping the guard to allow time to actually advance

				let _guard = timeout.await;
				assert_eq!(
					Duration::from_secs(seconds) + Duration::from_millis(500),
					timer_registry.now() - start
				);

				operation_count.fetch_add(1, Ordering::SeqCst);
			}
		}
	});

	assert_eq!(
		0,
		operation_count.load(Ordering::SeqCst),
		"Should not have performed an operation before advancing time"
	);

	timer_registry.advance_time(Duration::ZERO).await;
	assert_eq!(
		0,
		operation_count.load(Ordering::SeqCst),
		"Should not have performed operation immediately after tick finished"
	);

	timer_registry.advance_time(Duration::from_millis(500)).await;
	assert_eq!(
		1,
		operation_count.load(Ordering::SeqCst),
		"Should have performed operation once after tick + timeout"
	);

	timer_registry.advance_time(Duration::from_secs(1)).await;
	assert_eq!(
		2,
		operation_count.load(Ordering::SeqCst),
		"Should have performed operation twice one period after the first one"
	);

	timer_registry.advance_time(Duration::from_secs(8)).await;
	assert_eq!(
		10,
		operation_count.load(Ordering::SeqCst),
		"Should have performed the operation 10 times once the time is over"
	);

	timer_registry.advance_time(Duration::from_secs(1)).await;
	assert_eq!(
		10,
		operation_count.load(Ordering::SeqCst),
		"Should not have performed the operation more than 10 times"
	);

	join_handle.await.expect("Task performing operations crashed");
}

#[tokio::test]
async fn should_not_deadlock_with_interval_and_timeout_of_same_length() {
	// This is a regression test for an issue that happened in communityvi

	let timer_registry = Arc::new(TimerRegistry::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		async move {
			let mut interval = timer_registry.interval(Duration::from_secs(1));

			let (guard, _) = interval.tick().await;

			drop(guard);

			let _ = timer_registry
				.timeout(Duration::from_secs(1), std::future::pending::<()>())
				.await
				.expect_err("Should have timed out");
		}
	});

	timer_registry.advance_time(Duration::from_secs(2)).await;

	join_handle.await.expect("task crashed");
}
