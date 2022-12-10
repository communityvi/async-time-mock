use async_time_mock_core::TimerRegistry;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn should_schedule_periodic_timer() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let tick_count = Arc::new(AtomicUsize::default());

	let start = timer_registry.now();

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let tick_count = tick_count.clone();
		async move {
			let mut interval = timer_registry.interval(Duration::from_secs(1));
			for seconds in 0..10 {
				let _guard = interval.tick().await;
				tick_count.fetch_add(1, Ordering::SeqCst);
				assert_eq!(Duration::from_secs(seconds), timer_registry.now() - start);
			}
		}
	});

	assert_eq!(
		0,
		tick_count.load(Ordering::SeqCst),
		"Should not have ticked before advancing time"
	);
	timer_registry.advance_time(Duration::ZERO).await;
	assert_eq!(
		1,
		tick_count.load(Ordering::SeqCst),
		"Should have ticked once immediately."
	);

	timer_registry.advance_time(Duration::from_millis(500)).await;
	assert_eq!(
		1,
		tick_count.load(Ordering::SeqCst),
		"Should not have ticked again after half of period has passed."
	);

	timer_registry.advance_time(Duration::from_millis(500)).await;
	assert_eq!(
		2,
		tick_count.load(Ordering::SeqCst),
		"Should have ticked twice by the time the first period has passed"
	);

	timer_registry.advance_time(Duration::from_secs(2)).await;
	assert_eq!(
		4,
		tick_count.load(Ordering::SeqCst),
		"Should have ticked four times after two more periods"
	);

	timer_registry.advance_time(Duration::from_secs(1337)).await;
	assert_eq!(
		10,
		tick_count.load(Ordering::SeqCst),
		"Should have finished ticking after a long time has passed"
	);

	join_handle.await.expect("Interval task crashed");
}

#[tokio::test]
async fn should_schedule_periodic_timer_at() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let tick_count = Arc::new(AtomicUsize::default());

	let start = timer_registry.now();

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let tick_count = tick_count.clone();
		async move {
			let start = start + Duration::from_secs(42);
			let mut interval = timer_registry.interval_at(start, Duration::from_secs(1));
			for seconds in 0..10 {
				let _guard = interval.tick().await;
				tick_count.fetch_add(1, Ordering::SeqCst);
				assert_eq!(Duration::from_secs(seconds), timer_registry.now() - start);
			}
		}
	});

	assert_eq!(
		0,
		tick_count.load(Ordering::SeqCst),
		"Should not have ticked before advancing time"
	);
	timer_registry.advance_time(Duration::ZERO).await;
	assert_eq!(
		0,
		tick_count.load(Ordering::SeqCst),
		"Should not have ticked immediately."
	);

	timer_registry.advance_time(Duration::from_secs(1)).await;
	assert_eq!(
		0,
		tick_count.load(Ordering::SeqCst),
		"Should not have ticked after one period if the start time hasn't been reached yet.",
	);

	timer_registry.advance_time(Duration::from_secs(41)).await;
	assert_eq!(
		1,
		tick_count.load(Ordering::SeqCst),
		"Should have ticked once after reaching start time"
	);

	timer_registry.advance_time(Duration::from_secs(2)).await;
	assert_eq!(
		3,
		tick_count.load(Ordering::SeqCst),
		"Should have ticked three times after two more periods"
	);

	timer_registry.advance_time(Duration::from_secs(1337)).await;
	assert_eq!(
		10,
		tick_count.load(Ordering::SeqCst),
		"Should have finished ticking after a long time has passed"
	);

	join_handle.await.expect("Interval task crashed");
}
