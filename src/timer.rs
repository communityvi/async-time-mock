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

	pub(crate) async fn run(self) {
		let Self {
			trigger,
			handler_finished_waiter,
		} = self;

		trigger.notify(1);
		handler_finished_waiter.wait().await;
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
