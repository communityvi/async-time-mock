# Changelog for async-time-mock-smol

# 0.3.0

* Increase minimum rust version to `1.70`
* Add `MockableClock::system_time` to get the current `SystemTime`.
  See [#82](https://github.com/communityvi/async-time-mock/issues/82).
* Turn `MockableClock::sleep` and `MockableClock::sleep_until` from async functions into functions returning `impl Future` with a `Send` and `'static` future type.
  See [#81](https://github.com/communityvi/async-time-mock/issues/81).
* Remove unnecessary `async` from `MockableClock::interval` and `MockableClock::interval_at`.
  Fixing [#81](https://github.com/communityvi/async-time-mock/issues/81) along the way.

# 0.2.0
* Update `smol` to `2` (NOTE: We're not depending on that directly, see the dependencies below)
	* Update `event_listener` to `4`
	* Update `async-io` to `2`
* Update `async-time-mock-core` to `0.1.2`

# 0.1.0
* Implement `elapsed` on `Instant`

# 0.0.1
* First release of `async-time-mock-smol`
