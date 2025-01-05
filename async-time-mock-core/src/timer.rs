use crate::time_handler_guard::TimeHandlerFinished;
use crate::TimeHandlerGuard;
use event_listener::{Event, EventListener};

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
				handler_guard,
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

pub(crate) struct TimerListener {
	listener: EventListener,
	handler_guard: TimeHandlerGuard,
}

impl TimerListener {
	pub(crate) async fn wait_until_triggered(self) -> TimeHandlerGuard {
		let Self {
			listener,
			handler_guard,
		} = self;

		listener.await;
		handler_guard
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

		let mut wait_until_triggered = pin!(listener.wait_until_triggered());
		assert!(
			poll_once(wait_until_triggered.as_mut()).await.is_none(),
			"Future should have been pending before the timer is triggered",
		);
		let _ = timer.trigger();

		assert!(
			poll_once(wait_until_triggered.as_mut()).await.is_some(),
			"Future should have been ready after timer was triggered"
		);
	}

	#[tokio::test]
	async fn time_handler_finished_should_be_triggered_by_time_handler_completion() {
		let (timer, listener) = Timer::new();

		let time_handler_finished = timer.trigger();
		let time_handler_guard = listener.wait_until_triggered().await;

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
