use std::cell::UnsafeCell;
use std::future::Future;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU8, Ordering};
use std::task::{Context, Poll, Waker};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
	let channel = Box::new(Channel {
		state: AtomicU8::new(State::Empty.into()),
		value: UnsafeCell::new(MaybeUninit::uninit()),
		waker: UnsafeCell::new(MaybeUninit::uninit()),
	});

	let channel = NonNull::new(Box::into_raw(channel))
		.unwrap_or_else(|| unreachable!("It's impossible that the box contains a null pointer"));

	(Sender { channel }, Receiver { channel })
}

pub struct Sender<T> {
	channel: NonNull<Channel<T>>,
}

unsafe impl<T> Send for Sender<T> {}
unsafe impl<T> Sync for Sender<T> {}

impl<T> Sender<T> {
	pub fn send(self, value: T) -> Result<(), T> {
		let channel = unsafe { self.channel.as_ref() };
		// always write the value at first
		unsafe { (*channel.value.get()).write(value) };

		use State::*;
		// lock the channel
		let previous_state = channel.lock();

		let mut waker = None;
		let (result, new_state) = match previous_state {
			Empty => (Ok(()), Full),
			ReceiverClosed => {
				// take the value back out and return it
				(Err(unsafe {channel.take_value()}), ReceiverClosed)
			}
			Waiting => {
				waker = Some(unsafe {channel.take_waker()});

				(Ok(()), Full)
			},
			Full => unreachable!("It shouldn't be possible for the channel to be full since we are the only sender and didn't write to it yet"),
			SenderClosedEmpty | SenderClosedFull => unreachable!("It shouldn't be possible for the channel to be closed since we are the only sender and still alive"),
			Locked => unreachable!("We just ensured that we hold the lock so the previous state must not have been locked"),
		};

		channel.state.store(new_state.into(), Ordering::SeqCst);

		if let Some(waker) = waker {
			// NOTE: Waking the waker after the state has been unlocked.
			//       There are two reasons for this:
			//       1. The waker might panic and we don't want to leave the state locked in that case
			//       2. This way the state can never be locked when the receiver is polled after being woken
			//
			//       What if the receiver is polled again in the meantime but with a different waker?
			//       In that case we would spuriously wake the old waker, but the Receiver would still
			//       receive the value because once the state is unlocked, the state already contains
			//       the information that the value was sent and the Receiver can immediately return Poll::Ready
			//       without any further necessity for waking the new waker.
			waker.wake();
		}

		result
	}
}

impl<T> Drop for Sender<T> {
	fn drop(&mut self) {
		let previous_state = unsafe { self.channel.as_ref() }.lock();

		let mut waker = None;

		use State::*;
		let new_state = match previous_state {
			Empty => Some(SenderClosedEmpty),
			ReceiverClosed => None,
			Waiting => {
				waker = Some(unsafe { self.channel.as_ref().take_waker() });
				Some(SenderClosedEmpty)
			}
			Full => Some(SenderClosedFull),
			SenderClosedEmpty | SenderClosedFull => unreachable!(
				"It shouldn't be possible for the channel to be full and closed if the sender isn't dropped yet"
			),
			Locked => {
				unreachable!("We just ensured that we hold the lock so the previous state must not have been locked")
			}
		};

		match new_state {
			Some(new_state) => {
				unsafe { self.channel.as_ref() }
					.state
					.store(new_state.into(), Ordering::SeqCst);
			}
			None => {
				unsafe { std::ptr::drop_in_place(self.channel.as_ptr()) };
			}
		}

		if let Some(waker) = waker {
			// NOTE: Waking the waker after the state has been unlocked.
			//       There are two reasons for this:
			//       1. The waker might panic and we don't want to leave the state locked
			//       2. This way the state can never be locked when the receiver is polled after being woken
			waker.wake();
		}
	}
}

pub struct Receiver<T> {
	channel: NonNull<Channel<T>>,
}

unsafe impl<T> Send for Receiver<T> {}
unsafe impl<T> Sync for Receiver<T> {}

impl<T> Future for Receiver<T> {
	type Output = Option<T>;

	fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();

		let channel = unsafe { this.channel.as_ref() };

		let previous_state = channel.lock();

		use State::*;
		let (result, new_state) = match previous_state {
			Empty => {
				// FIXME: Take panics into account
				unsafe { (*channel.waker.get()).write(context.waker().clone()) };
				(Poll::Pending, Waiting)
			}
			SenderClosedEmpty => (Poll::Ready(None), SenderClosedEmpty),
			Waiting => {
				let waker = unsafe { channel.waker_mut() };
				// FIXME: Take panics into account
				waker.clone_from(context.waker());
				(Poll::Pending, Waiting)
			}
			Full => {
				let value = unsafe { channel.take_value() };
				(Poll::Ready(Some(value)), ReceiverClosed)
			}
			Locked => {
				unreachable!("We just ensured that we hold the lock so the previous state must not have been locked")
			}
			ReceiverClosed => unreachable!("We are the receiver and are still alive so this can't happen"),
			SenderClosedFull => {
				let value = unsafe { channel.take_value() };
				(Poll::Ready(Some(value)), SenderClosedEmpty)
			}
		};

		channel.state.store(new_state.into(), Ordering::SeqCst);

		result
	}
}

impl<T> Drop for Receiver<T> {
	fn drop(&mut self) {
		let previous_state = unsafe { self.channel.as_ref() }.lock();

		// NOTE: Storing waker and value that need to be dropped in here such that the drop is called
		//       after the state has been unlocked successfully. This ensures that the state won't stay
		//       locked if any of the drops panic
		let mut _waker_to_drop = None;
		let mut _value_to_drop = None;

		use State::*;
		let new_state = match previous_state {
			Empty => Some(ReceiverClosed),
			SenderClosedEmpty => None,
			Waiting => {
				_waker_to_drop = Some(unsafe { self.channel.as_ref().take_waker() });
				Some(ReceiverClosed)
			}
			Full => {
				_value_to_drop = Some(unsafe { self.channel.as_ref().take_value() });
				Some(ReceiverClosed)
			}
			SenderClosedFull => {
				_value_to_drop = Some(unsafe { self.channel.as_ref().take_value() });
				None
			}
			Locked => {
				unreachable!("We just ensured that we hold the lock so the previous state must not have been locked")
			}
			ReceiverClosed => Some(ReceiverClosed),
		};

		match new_state {
			Some(new_state) => {
				unsafe { self.channel.as_ref() }
					.state
					.store(new_state.into(), Ordering::SeqCst);
			}
			None => {
				unsafe { std::ptr::drop_in_place(self.channel.as_ptr()) };
			}
		}
	}
}

struct Channel<T> {
	state: AtomicU8,
	value: UnsafeCell<MaybeUninit<T>>,
	waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl<T> Channel<T> {
	/// Lock the channel, returning the previous state
	fn lock(&self) -> State {
		loop {
			// TODO: Figure out correct memory ordering
			let previous = self.state.swap(State::Locked.into(), Ordering::SeqCst);
			let previous = unsafe { State::from_u8(previous) };
			if previous == State::Locked {
				std::hint::spin_loop();
				continue;
			}

			break previous;
		}
	}

	/// SAFETY: The caller must ensure that the channel is locked and that the waker is set
	unsafe fn take_waker(&self) -> Waker {
		let maybe_uninit = std::mem::replace(unsafe { &mut *self.waker.get() }, MaybeUninit::uninit());
		unsafe { maybe_uninit.assume_init() }
	}

	/// SAFETY: The caller must ensure that the channel is locked and that the waker is set
	#[allow(clippy::mut_from_ref)]
	unsafe fn waker_mut(&self) -> &mut Waker {
		unsafe { (*self.waker.get()).assume_init_mut() }
	}

	/// SAFETY: The caller must ensure that the channel is locked and that the value is set
	unsafe fn take_value(&self) -> T {
		let maybe_uninit = std::mem::replace(unsafe { &mut *self.value.get() }, MaybeUninit::uninit());
		unsafe { maybe_uninit.assume_init() }
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum State {
	Empty,
	ReceiverClosed,
	SenderClosedEmpty,
	SenderClosedFull,
	Waiting,
	Full,
	Locked,
}

impl State {
	/// SAFETY: The state `u8` passed in must previously have been created from a `State`.
	unsafe fn from_u8(state: u8) -> Self {
		unsafe { std::mem::transmute(state) }
	}
}

impl From<State> for u8 {
	fn from(state: State) -> Self {
		state as u8
	}
}
