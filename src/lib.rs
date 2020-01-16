//! Provides a
//! [`Stream`](../futures/stream/trait.Stream.html) combinator, to limit the rate at which
//! items are produced.
//!
//! ## Key Features
//! - Throttling is implemented via
//! [`poll()`](../futures/future/trait.Future.html#tymethod.poll), and not via any sort of
//! buffering.
//! - The throttling behaviour can be applied to both `Stream`'s and `Future`'s.
//! - Multiple streams/futures can be throttled together as a group.
//!
//! ## Example throttling of `Stream`
//! ```no_run
//! # use futures::prelude::*;
//! # use futures::stream;
//! # use std::time::Duration;
//! # use stream_throttle::{ThrottlePool, ThrottleRate, ThrottledStream};
//! #
//! #[tokio::main]
//! async fn main() {
//!     let rate = ThrottleRate::new(5, Duration::new(1, 0));
//!     let pool = ThrottlePool::new(rate);
//!
//!     let work = stream::repeat(())
//!       .throttle(pool)
//!       .take(10)
//!       .collect::<Vec<_>>()
//!       .await;
//! }
//! ```
//!
//! ## Example throttling of `Future`
//! ```no_run
//! # use futures::prelude::*;
//! # use std::time::Duration;
//! # use stream_throttle::{ThrottlePool, ThrottleRate, ThrottledStream};
//! #
//! #[tokio::main]
//! async fn main() {
//!     let rate = ThrottleRate::new(5, Duration::new(1, 0));
//!     let pool = ThrottlePool::new(rate);
//!
//!     let work = pool.queue().await;
//! }
//! ```

pub mod error;
mod pool;
mod rate;
mod stream;

pub use pool::*;
pub use rate::*;
pub use stream::*;
