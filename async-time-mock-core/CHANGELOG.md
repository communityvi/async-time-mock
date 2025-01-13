# Changelog for async-time-mock-core

# 0.1.4

* Increase minimum rust version to `1.70`
* `TimerRegistry::sleep` and `TimerRegistry::sleep_until` now return `TimerListener` as an explicit future type instead
  of `impl Future`
* Add `TimerRegistry::system_time` to get a mocked `SystemTime` in addition to the mocked monotonic time.
* Derive `Debug` on `TimeHandlerGuard`
* Replace `futures-lite::pin` with `std::pin::pin` (tests only)

# 0.1.3

* Update `event_listener` to `5`

# 0.1.2

* Update `event_listener` to `4`

# 0.1.1

* Update `async-lock` to `3`
* Update `event-listener` to `3`
* Update `futures-lite` to `2`

# 0.1.0

* Implement `elapsed` on `Instant`

# 0.0.1

* First release of `async-time-mock-core`
