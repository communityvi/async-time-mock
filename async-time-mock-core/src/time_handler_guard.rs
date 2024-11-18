use crate::oneshot;

#[must_use = "TimeHandlerGuard must be kept until the timer has performed it's side-effects"]
pub struct TimeHandlerGuard(Option<oneshot::Sender<()>>);

impl TimeHandlerGuard {
	pub(crate) fn new() -> (Self, TimeHandlerFinished) {
		let (sender, receiver) = oneshot::channel();
		(Self(Some(sender)), TimeHandlerFinished(receiver))
	}
}

impl Drop for TimeHandlerGuard {
	fn drop(&mut self) {
		if let Some(sender) = self.0.take() {
			let _ = sender.send(());
		}
	}
}

pub(crate) struct TimeHandlerFinished(oneshot::Receiver<()>);

impl TimeHandlerFinished {
	pub(crate) async fn wait(self) {
		self.0.await;
	}
}

#[cfg(test)]
mod test {
	use crate::TimeHandlerGuard;
	use futures_lite::future::poll_once;
	use std::pin::pin;

	#[tokio::test]
	async fn should_notify_once_time_handler_guard_is_dropped() {
		let (guard, waiter) = TimeHandlerGuard::new();

		let mut waiter_future = pin!(waiter.wait());
		assert!(
			poll_once(waiter_future.as_mut()).await.is_none(),
			"Waiter should have been pending before the guard is dropped",
		);

		drop(guard);
		assert!(
			poll_once(waiter_future.as_mut()).await.is_some(),
			"Waiter should have been ready after the guard was dropped",
		);
	}
}
