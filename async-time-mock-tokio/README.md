# async-time-mock-tokio

Asynchronous time mocking with a tokio compatible API based on [async-time-mock-core](https://github.com/communityvi/async-time-mock/tree/main/async-time-mock-core), inspired by the approach described in [Mocking Time In Async Rust](https://www.ditto.live/blog/mocking-time-in-async-rust).

NOTE: This library is still in it's infancy, the API is still likely to change (read: improve). Please leave your feedback and suggestions on [GitHub](https://github.com/communityvi/async-time-mock).

## Cargo features
* `mock`: Enable the mock clock. If you only enable this in tests, this library turns into a thin wrapper around tokio's time functions.

## Example

```rust
use async_time_mock_tokio::MockableClock;
use std::{
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};

static HAS_SLEPT: AtomicBool = AtomicBool::new(false);

async fn sleep(clock: MockableClock) {
	// Sleep is either mocked or a real tokio::sleep, depending on which variant of `MockableClock` you pass in.
	let _guard = clock.sleep(Duration::from_secs(3600)).await;
	// Dropping this guard signifies that all the effects of the timer have finished.
	// This allows test code to wait until the condition to assert for has happened.

	println!("Slept for an hour");
	HAS_SLEPT.store(true, Ordering::SeqCst);
}

#[tokio::main]
async fn main() {
	let (clock, controller) = MockableClock::mock(); // In production, you can use MockableClock::Real instead

	tokio::spawn(sleep(clock));

	controller.advance_time(Duration::from_secs(600)).await;
	assert!(!HAS_SLEPT.load(Ordering::SeqCst), "Timer won't trigger after just 10 minutes.");

	// advance_time will first trigger the sleep in the task above and then wait until the `_guard` was dropped.
	// This ensures that the task had enough time to actually set `HAS_SLEPT` to `true`.
	controller.advance_time(Duration::from_secs(3000)).await;
	assert!(HAS_SLEPT.load(Ordering::SeqCst), "Timer has triggered after 1 hour.")
}
```

