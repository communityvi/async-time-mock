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
	use crate::TimeHandlerGuard;
	use futures_util::poll;

	#[tokio::test]
	async fn should_notify_once_time_handler_guard_is_dropped() {
		let (guard, waiter) = TimeHandlerGuard::new();

		let mut waiter_future = Box::pin(waiter.wait());
		assert!(
			poll!(waiter_future.as_mut()).is_pending(),
			"Waiter should have been pending before the guard is dropped"
		);

		drop(guard);
		waiter_future.await;
	}
}
