use tokio::sync::oneshot;

#[must_use = "TimeHandlerGuard must be kept until the timer has performed it's side-effects"]
pub struct TimeHandlerGuard(Option<oneshot::Sender<()>>);

impl TimeHandlerGuard {
	pub fn new() -> (Self, oneshot::Receiver<()>) {
		let (sender, receiver) = oneshot::channel();
		(Self(Some(sender)), receiver)
	}
}

impl Drop for TimeHandlerGuard {
	fn drop(&mut self) {
		if let Some(sender) = self.0.take() {
			// ignore error, because that means the other end is dropped anyways
			let _ = sender.send(());
		}
	}
}
