use crate::time_handler_guard::TimeHandlerFinished;
use crate::TimeHandlerGuard;
use event_listener::{Event, EventListener};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

pub(crate) struct Timer {
	trigger: Event,
	handler_finished_waiter: TimeHandlerFinished,
}

impl Timer {
	pub(crate) fn new() -> (Self, TimerListener) {
		let (handler_guard, handler_finished_waiter) = TimeHandlerGuard::new();
		let trigger = Event::new();
		let listener = trigger.listen();
		(
			Self {
				trigger,
				handler_finished_waiter,
			},
			TimerListener {
				listener,
				handler_guard: Some(handler_guard),
			},
		)
	}

	pub(crate) fn trigger(self) -> TimeHandlerFinished {
		let Self {
			trigger,
			handler_finished_waiter,
		} = self;
		trigger.notify(1);
		handler_finished_waiter
	}
}

pin_project! {
	pub struct TimerListener {
		#[pin]
		listener: EventListener,
		handler_guard: Option<TimeHandlerGuard>,
	}
}

impl Future for TimerListener {
	type Output = TimeHandlerGuard;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.project();

		ready!(this.listener.poll(context));

		match this.handler_guard.take() {
			Some(handler_guard) => Poll::Ready(handler_guard),
			None => Poll::Pending,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_lite::future::poll_once;
	use std::pin::pin;

	#[tokio::test]
	async fn timer_should_trigger_timer_listener() {
		let (timer, listener) = Timer::new();

		let mut listener = pin!(listener);
		assert!(
			poll_once(listener.as_mut()).await.is_none(),
			"Future should have been pending before the timer is triggered",
		);
		let _ = timer.trigger();

		assert!(
			poll_once(listener.as_mut()).await.is_some(),
			"Future should have been ready after timer was triggered"
		);
	}

	#[tokio::test]
	async fn time_handler_finished_should_be_triggered_by_time_handler_completion() {
		let (timer, listener) = Timer::new();

		let time_handler_finished = timer.trigger();
		let time_handler_guard = listener.await;

		let mut waiter = pin!(time_handler_finished.wait());
		assert!(
			poll_once(waiter.as_mut()).await.is_none(),
			"Future should have been pending before the time handler is finished (guard dropped)",
		);

		drop(time_handler_guard);
		assert!(
			poll_once(waiter.as_mut()).await.is_some(),
			"Future should have been ready after the time handler is finished (guard dropped)",
		);
	}
}
