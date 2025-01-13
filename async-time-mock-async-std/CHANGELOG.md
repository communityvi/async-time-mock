# Changelog for async-time-mock-async-std

# 0.1.2
* Increase minimum rust version to `1.70`
* Add `MockableClock::system_time` to get the current `SystemTime`.
  See [#82](https://github.com/communityvi/async-time-mock/issues/82).
* Turn `MockableClock::sleep` and `MockableClock::sleep_until` from async functions into functions returning `impl Future` with a `Send` and `'static` future type.
  See [#81](https://github.com/communityvi/async-time-mock/issues/81).

# 0.1.1
* Change `async-time-mock-core` dependency to `0.1`

# 0.1.0
* Implement `elapsed` on `Instant`

# 0.0.1
* First release of `async-time-mock-async-std`
