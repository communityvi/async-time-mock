use event_listener::{Event, EventListener};

#[must_use = "TimeHandlerGuard must be kept until the timer has performed it's side-effects"]
pub struct TimeHandlerGuard(Event);

impl TimeHandlerGuard {
	pub(crate) fn new() -> (Self, TimeHandlerFinished) {
		let event = Event::new();
		let listener = event.listen();
		(Self(event), TimeHandlerFinished(listener))
	}
}

impl Drop for TimeHandlerGuard {
	fn drop(&mut self) {
		self.0.notify(1);
	}
}

pub(crate) struct TimeHandlerFinished(EventListener);

impl TimeHandlerFinished {
	pub(crate) async fn wait(self) {
		self.0.await
	}
}

#[cfg(test)]
mod test {
	use crate::{tokio_test, TimeHandlerGuard};
	use futures_lite::future::poll_once;
	use futures_lite::pin;

	#[test]
	fn should_notify_once_time_handler_guard_is_dropped() {
		tokio_test!({
			let (guard, waiter) = TimeHandlerGuard::new();

			let waiter_future = waiter.wait();
			pin!(waiter_future);
			assert!(
				poll_once(waiter_future.as_mut()).await.is_none(),
				"Waiter should have been pending before the guard is dropped",
			);

			drop(guard);
			assert!(
				poll_once(waiter_future.as_mut()).await.is_some(),
				"Waiter should have been ready after the guard was dropped",
			);
		});
	}
}
