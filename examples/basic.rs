use futures::prelude::*;
use pin_utils::pin_mut;
use std::time::{Duration, Instant};
use stream_throttle::{ThrottlePool, ThrottleRate, ThrottledStream};

#[tokio::main]
async fn main() {
	let rate = ThrottleRate::new(5, Duration::new(1, 0));
	println!("{:?}", rate);

	let pool = ThrottlePool::new(rate);

	let stream1 = stream::repeat(()).throttle(pool.clone()).take(10).fuse();
	let stream2 = stream::repeat(()).throttle(pool.clone()).take(10).fuse();
	let stream3 = stream::repeat(()).throttle(pool.clone()).take(10).fuse();

	pin_mut!(stream1);
	pin_mut!(stream2);
	pin_mut!(stream3);

	let mut last_instant = Instant::now();

	loop {
		futures::select! {
			_ = stream1.select_next_some() => {
				print!("(stream 1)");
			}
			_ = stream2.select_next_some() => {
				print!("(stream 2)");
			}
			_ = stream3.select_next_some() => {
				print!("(stream 3)");
			}
			complete => {
				break;
			}
		}

		println!(" item delayed: {:?}", last_instant.elapsed());
		last_instant = Instant::now();
	}
}
