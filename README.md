# async-time-mock

Mockable time for use in async runtimes.

Based on the approach described in https://www.ditto.live/blog/mocking-time-in-async-rust

WARNING: The API is far from stable, we're still focussing on features without full attention on getting the API right. This API WILL BREAK.

Supported runtimes:
* [`tokio`](https://github.com/tokio-rs/tokio)
* [`smol`](https://github.com/smol-rs/smol)
* [`async-std`](https://github.com/async-rs/async-std)
* nothing: You can also use just mocks, without any runtime 😉

Supported operations:
* `sleep`
