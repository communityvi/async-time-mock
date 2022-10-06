use crate::TimeHandlerGuard;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Timeout<F> {
	future: F,
	sleep: Pin<Box<dyn Future<Output = TimeHandlerGuard> + Send>>,
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
		// SAFETY: `this` is never used to move the underlying data.
		let this = unsafe { self.get_unchecked_mut() };
		use Poll::*;
		if let Ready(guard) = this.sleep.as_mut().poll(context) {
			return Ready(Err(Elapsed(guard)));
		};

		// SAFETY: `this` comes from a `self: Pin<&mut Self>` therefore `this.future` is already
		// transitively pinned.
		let pinned_future = unsafe { Pin::new_unchecked(&mut this.future) };
		pinned_future.poll(context).map(Ok)
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
