use crate::TimeHandlerGuard;
use pin_project_lite::pin_project;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
	pub struct Timeout<F> {
		#[pin]
		future: F,
		sleep: Pin<Box<dyn Future<Output = TimeHandlerGuard> + Send>>,
	}
}

impl<F> Timeout<F> {
	pub(crate) fn new(future: F, sleep: impl Future<Output = TimeHandlerGuard> + Send + 'static) -> Self {
		let sleep = Box::pin(sleep);
		Self { sleep, future }
	}

	pub fn get_ref(&self) -> &F {
		&self.future
	}

	pub fn get_mut(&mut self) -> &mut F {
		&mut self.future
	}

	pub fn into_inner(self) -> F {
		self.future
	}
}

impl<F> Future for Timeout<F>
where
	F: Future,
{
	type Output = Result<F::Output, Elapsed>;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.project();
		use Poll::*;
		if let Ready(guard) = this.sleep.as_mut().poll(context) {
			return Ready(Err(Elapsed(guard)));
		};

		this.future.poll(context).map(Ok)
	}
}

#[must_use = "Elapsed must be kept until the timer has performed it's side-effects"]
pub struct Elapsed(pub TimeHandlerGuard);

impl Display for Elapsed {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("Timeout elapsed.")
	}
}

impl Debug for Elapsed {
	fn fmt(&self, format: &mut Formatter<'_>) -> std::fmt::Result {
		format.debug_tuple("Elapsed").finish()
	}
}
