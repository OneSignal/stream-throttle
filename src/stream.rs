use crate::ThrottlePool;
use futures::ready;
use futures::{prelude::*, task::*};
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use std::pin::Pin;

/// Provides a `throttle()` method on all `Stream`'s.
pub trait ThrottledStream {
	/// Returns a new stream, which throttles items from the original stream, according to the
	/// rate defined by `pool`.
	fn throttle(self, pool: ThrottlePool) -> Throttled<Self>
	where
		Self: Stream + Unpin + Sized,
	{
		Throttled {
			stream: self,
			pool,
			pending: None,
		}
	}
}

impl<T: Stream> ThrottledStream for T {}

/// A stream combinator which throttles its elements via a shared `ThrottlePool`.
///
/// This structure is produced by the `ThrottledStream::throttle()` method.
#[must_use = "streams do nothing unless polled"]
pub struct Throttled<S>
where
	S: Stream,
{
	stream: S,
	pool: ThrottlePool,

	// the first Option layer represents a pending item for this Throttled stream
	// The second Option layer contains a future produced by ThrottlePool::queue()
	pending: Option<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl<S> Unpin for Throttled<S> where S: Stream + Unpin {}

impl<S> Throttled<S>
where
	S: Stream,
{
	// To make using this macro safe, three things need to be ensured:
	//
	// - If the struct implements Drop, the drop method is not allowed to move
	//   the value of the field.
	// - If the struct wants to implement Unpin, it has to do so conditionally:
	//   The struct can only implement Unpin if the field's type is Unpin.
	// - The struct must not be #[repr(packed)].
	unsafe_pinned!(stream: S);
	unsafe_unpinned!(pending: Option<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>);
}

impl<S> Stream for Throttled<S>
where
	S: Stream,
{
	type Item = S::Item;

	/// Calls ThrottlePool::queue() to get slot in the throttle queue, waits for it to resolve, and
	/// then polls the underlying stream for an item, and produces it.
	fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<S::Item>> {
		// if we don't already have a pending item, get one
		if self.pending.is_none() {
			*(self.as_mut().pending()) = Some(Some(Box::pin(self.pool.queue())));
		}
		assert!(self.pending.is_some());

		// if we have a queue() future, we are still waiting for the queue slot to expire
		if let Some(Some(pending)) = self.as_mut().pending().as_mut() {
			// poll the queue slot future, and remove it once it resolves
			ready!(pending.as_mut().poll(ctx));
			self.as_mut().pending().as_mut().unwrap().take();
		}

		// poll the underlying stream until we get an item, or the stream ends
		match ready!(self.as_mut().stream().poll_next(ctx)) {
			Some(item) => {
				*(self.as_mut().pending()) = None;
				Poll::Ready(Some(item))
			}

			None => Poll::Ready(None),
		}
	}
}
