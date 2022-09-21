use async_time_mock_core::TimerRegistry;
use futures_lite::future::poll_once;
use futures_lite::pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::join;

#[tokio::test]
#[should_panic]
async fn sleep_should_panic_with_zero_duration() {
	let timer_registry = TimerRegistry::default();
	let _ = timer_registry.sleep(Duration::ZERO).await;
}

#[tokio::test]
async fn sleep_should_finish_if_time_is_advanced_by_exactly_sleep_amount() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let has_slept = Arc::new(AtomicBool::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_slept = has_slept.clone();
		async move {
			let _guard = timer_registry.sleep(Duration::from_secs(10)).await;
			has_slept.store(true, Ordering::SeqCst);
			assert_eq!(Duration::from_secs(10), timer_registry.now() - start);
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

#[tokio::test]
async fn sleep_should_not_finish_if_the_time_is_advanced_by_less_than_sleep_amount() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let has_slept = Arc::new(AtomicBool::default());

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_slept = has_slept.clone();
		async move {
			let _guard = timer_registry.sleep(Duration::from_secs(10)).await;
			has_slept.store(true, Ordering::SeqCst);
			assert_eq!(Duration::from_secs(10), timer_registry.now() - start);
		}
	});

	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should not have slept before the time has advanced"
	);
	timer_registry.advance_time(Duration::from_secs(5)).await;
	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should not have slept before the full sleep time has been reached (1)",
	);
	timer_registry.advance_time(Duration::from_secs(4)).await;
	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should not have slept before the full sleep time has been reached (2)",
	);
	timer_registry.advance_time(Duration::from_secs(1)).await;
	assert!(
		has_slept.load(Ordering::SeqCst),
		"Should have slept after the full sleep time has been reached",
	);

	join_handle.await.expect("Sleeping task crashed");
}

#[tokio::test]
async fn should_work_with_multiple_sleeps_of_same_length() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let sleep_counter = Arc::new(AtomicUsize::default());

	let sleep_handles = (0..10)
		.into_iter()
		.map(|_| {
			let timer_registry = timer_registry.clone();
			let sleep_counter = sleep_counter.clone();
			tokio::task::spawn(async move {
				let _guard = timer_registry.sleep(Duration::from_secs(10)).await;
				sleep_counter.fetch_add(1, Ordering::SeqCst);
				assert_eq!(Duration::from_secs(10), timer_registry.now() - start);
			})
		})
		.collect::<Vec<_>>();

	assert_eq!(
		0,
		sleep_counter.load(Ordering::SeqCst),
		"No timer should have been triggered before advancing time"
	);
	timer_registry.advance_time(Duration::from_secs(10)).await;
	assert_eq!(
		10,
		sleep_counter.load(Ordering::SeqCst),
		"All timers should have been triggered after advancing time"
	);

	for sleep_handle in sleep_handles {
		sleep_handle.await.expect("Sleeping task crashed");
	}
}

#[tokio::test]
async fn should_work_with_multiple_sleeps_of_different_length_all_at_once() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let sleep_counter = Arc::new(AtomicUsize::default());

	let sleep_handles = (1..=10)
		.rev()
		.map(|seconds| {
			let timer_registry = timer_registry.clone();
			let sleep_counter = sleep_counter.clone();
			tokio::task::spawn(async move {
				let _guard = timer_registry.sleep(Duration::from_secs(seconds)).await;
				sleep_counter.fetch_add(1, Ordering::SeqCst);
				assert_eq!(Duration::from_secs(seconds), timer_registry.now() - start);
			})
		})
		.collect::<Vec<_>>();

	assert_eq!(
		0,
		sleep_counter.load(Ordering::SeqCst),
		"No timer should have been triggered before advancing time"
	);
	timer_registry.advance_time(Duration::from_secs(10)).await;
	assert_eq!(
		10,
		sleep_counter.load(Ordering::SeqCst),
		"All timers should have been triggered after advancing time"
	);

	for sleep_handle in sleep_handles {
		sleep_handle.await.expect("Sleeping task crashed");
	}
}

#[tokio::test]
async fn should_work_with_multiple_sleeps_of_different_length_in_steps() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let sleep_counter = Arc::new(AtomicUsize::default());

	let sleep_handles = (1..=10)
		.rev()
		.map(|seconds| {
			let timer_registry = timer_registry.clone();
			let sleep_counter = sleep_counter.clone();
			tokio::task::spawn(async move {
				let _guard = timer_registry.sleep(Duration::from_secs(seconds)).await;
				sleep_counter.fetch_add(1, Ordering::SeqCst);
				assert_eq!(Duration::from_secs(seconds), timer_registry.now() - start);
			})
		})
		.collect::<Vec<_>>();

	assert_eq!(
		0,
		sleep_counter.load(Ordering::SeqCst),
		"No timer should have been triggered before advancing time"
	);
	timer_registry.advance_time(Duration::from_secs(3)).await;
	assert_eq!(
		3,
		sleep_counter.load(Ordering::SeqCst),
		"3 timers should have been triggered after advancing time by 3 seconds"
	);
	timer_registry.advance_time(Duration::from_secs(6)).await;
	assert_eq!(
		9,
		sleep_counter.load(Ordering::SeqCst),
		"9 timers should have been triggered after advancing time by 9 seconds"
	);
	timer_registry.advance_time(Duration::from_secs(1)).await;
	assert_eq!(
		10,
		sleep_counter.load(Ordering::SeqCst),
		"All timers should have been triggered after advancing time by 10 seconds"
	);

	for sleep_handle in sleep_handles {
		sleep_handle.await.expect("Sleeping task crashed");
	}
}

#[tokio::test]
async fn should_only_advance_time_once_the_first_timer_was_scheduled() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();

	let advance_time_future = timer_registry.advance_time(Duration::from_secs(1));
	pin!(advance_time_future);
	assert!(
		poll_once(advance_time_future.as_mut()).await.is_none(),
		"Advance time should still be waiting for a timer to be scheduled"
	);

	tokio::task::spawn({
		let timer_registry = timer_registry.clone();
		async move {
			let _guard = timer_registry.sleep(Duration::from_secs(10)).await;
		}
	});

	advance_time_future.await;
	assert_eq!(
		Duration::from_secs(1),
		timer_registry.now() - start,
		"Did not advance time after scheduling timer"
	);
}

#[tokio::test]
async fn should_sleep_until() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let has_slept = Arc::new(AtomicBool::default());
	let start = timer_registry.now();

	let until = start + Duration::from_secs(1337);

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_slept = has_slept.clone();
		async move {
			let _guard = timer_registry.sleep_until(until).await;
			has_slept.store(true, Ordering::SeqCst);
			assert_eq!(Duration::from_secs(1337), timer_registry.now() - start);
		}
	});

	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should have slept before until has been reached"
	);
	timer_registry.advance_time(Duration::from_secs(1337)).await;
	assert_eq!(until, timer_registry.now());
	assert!(
		has_slept.load(Ordering::SeqCst),
		"Should have slept after until has been reached"
	);

	join_handle.await.expect("Sleeping task crashed");
}

#[tokio::test]
async fn sleep_until_should_resolve_after_zero_time_if_in_the_past() {
	let timer_registry = Arc::new(TimerRegistry::default());
	let start = timer_registry.now();
	let _ = join!(timer_registry.advance_time(Duration::from_secs(1337)), async {
		let _ = timer_registry.sleep(Duration::from_secs(1)).await;
	});

	let has_slept = Arc::new(AtomicBool::default());

	let until = timer_registry.now() - Duration::from_secs(42);

	let join_handle = tokio::spawn({
		let timer_registry = timer_registry.clone();
		let has_slept = has_slept.clone();
		async move {
			let _guard = timer_registry.sleep_until(until).await;
			has_slept.store(true, Ordering::SeqCst);
			assert_eq!(Duration::from_secs(1337), timer_registry.now() - start);
		}
	});

	assert!(
		!has_slept.load(Ordering::SeqCst),
		"Should have slept before advancing time"
	);
	timer_registry.advance_time(Duration::ZERO).await;
	assert!(
		has_slept.load(Ordering::SeqCst),
		"Should have slept after until has been reached"
	);

	join_handle.await.expect("Sleeping task crashed");
}
