# async-time-mock-async-std (discontinued)

# NOTE: This library is discontinued because `async-std` [was discontinued](https://github.com/async-rs/async-std/releases/tag/v1.13.1). You can find [`async-time-mock-smol`](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-smol) for an implementation with the `smol` runtime instead.

Asynchronous time mocking with an async-std compatible API based on [async-time-mock-core](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-core), inspired by the approach described in [Mocking Time In Async Rust](https://www.ditto.live/blog/mocking-time-in-async-rust).

NOTE: This library is still in it's infancy, the API is still likely to change (read: improve). Please leave your feedback and suggestions on [GitHub](https://github.com/communityvi/async-time-mock).

## Cargo features
* `mock`: Enable the mock clock. If you only enable this in tests, this library turns into a thin wrapper around async-std's time functions.
* `stream`: Implement `futures_core::stream::Stream` for `Interval`

## Example

```rust
use async_time_mock_async_std::MockableClock;
use std::{
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};

static HAS_SLEPT: AtomicBool = AtomicBool::new(false);

async fn sleep(clock: MockableClock) {
	// Sleep is either mocked or a real async_std::task::sleep, depending on which variant of `MockableClock` you pass in.
	let _guard = clock.sleep(Duration::from_secs(3600)).await;
	// Dropping this guard signifies that all the effects of the timer have finished.
	// This allows test code to wait until the condition to assert for has happened.

	println!("Slept for an hour");
	HAS_SLEPT.store(true, Ordering::SeqCst);
}

#[async_std::main]
async fn main() {
	let (clock, controller) = MockableClock::mock(); // In production, you can use MockableClock::Real instead

	async_std::task::spawn(sleep(clock));

	controller.advance_time(Duration::from_secs(600)).await;
	assert!(!HAS_SLEPT.load(Ordering::SeqCst), "Timer won't trigger after just 10 minutes.");

	// advance_time will first trigger the sleep in the task above and then wait until the `_guard` was dropped.
	// This ensures that the task had enough time to actually set `HAS_SLEPT` to `true`.
	controller.advance_time(Duration::from_secs(3000)).await;
	assert!(HAS_SLEPT.load(Ordering::SeqCst), "Timer has triggered after 1 hour.")
}
```

