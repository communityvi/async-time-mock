# async-time-mock

Mockable time for use in async runtimes based on the approach described in [Mocking Time In Async Rust](https://www.ditto.live/blog/mocking-time-in-async-rust).

NOTE: This library is still in it's infancy, the API is still likely to change (read: improve). Please leave your feedback and suggestions on [GitHub](https://github.com/communityvi/async-time-mock).

See the following READMEs for further information.
* [async-time-mock-tokio](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-tokio): Support for the [tokio runtime](https://github.com/tokio-rs/tokio).
* [async-time-mock-async-std](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-async-std): Support for the [async-std runtime](https://github.com/async-rs/async-std).
* [async-time-mock-smol](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-smol): Support for the  [smol runtime](https://github.com/smol-rs/smol).
* [async-time-mock-core](https://github.com/communityvi/async-time-mock/tree/master/async-time-mock-core): Core primitives. Can be used to build support for more runtimes.
