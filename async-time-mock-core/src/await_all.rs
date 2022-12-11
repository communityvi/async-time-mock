use pin_project_lite::pin_project;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Await all given futures concurrently. Finishes once all futures are finished.
/// This is basically a simplified version of [`futures_util::futrure::join_all`].
/// That could have been used, but it would introduce a dependency to [`futures_util`].
pub(crate) fn await_all<FUTURE>(futures: impl IntoIterator<Item = FUTURE>) -> AwaitAll<FUTURE> {
	let futures = futures.into_iter().collect::<Vec<_>>().into_boxed_slice();
	let completed_indices = HashSet::with_capacity(futures.len());
	AwaitAll {
		futures,
		completed_indices,
	}
}

pin_project! {
	pub(crate) struct AwaitAll<FUTURE>
	{
		#[pin]
		futures: Box<[FUTURE]>,
		completed_indices: HashSet<usize>,
	}
}

impl<FUTURE> Future for AwaitAll<FUTURE>
where
	FUTURE: Future,
{
	type Output = ();

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.project();

		let mut futures: Pin<&mut Box<[FUTURE]>> = this.futures;
		for (index, future) in futures.iter_mut().enumerate() {
			if this.completed_indices.contains(&index) {
				continue;
			}

			// SAFETY: The future is never moved or replaced and it is also pinned as part of the AwaitAll itself
			let pinned_future = unsafe { Pin::new_unchecked(future) };
			if pinned_future.poll(context).is_ready() {
				this.completed_indices.insert(index);
			}
		}

		let length = futures.len();
		if this.completed_indices.len() < length {
			Poll::Pending
		} else {
			Poll::Ready(())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::await_all::await_all;
	use futures_lite::future::poll_once;
	use futures_lite::pin;
	use std::future;
	use std::future::Future;
	use std::pin::Pin;

	#[tokio::test]
	async fn should_run_all_futures() {
		await_all(vec![future::ready(()); 10]).await;
	}

	#[tokio::test]
	async fn should_be_pending_if_one_future_is() {
		let futures = vec![
			Box::pin(future::ready(())) as Pin<Box<dyn Future<Output = ()>>>,
			Box::pin(future::pending()),
		];

		let await_all = await_all(futures);
		pin!(await_all);
		assert!(
			poll_once(await_all.as_mut()).await.is_none(),
			"Future should have been pending"
		);
	}
}
