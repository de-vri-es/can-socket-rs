use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::{oneshot, mpsc};

use crate::{CanFilter, CanFrame};
use super::CanSocket;

#[derive(Clone)]
pub struct Router {
	inner: Arc<RouterInner>,
}

impl Router {
	async fn recv_frame(&self, deadline: Instant, filter: CanFilter) -> std::io::Result<Option<CanFrame>> {
		let mut subscriptions = self.inner.subscriptions.lock().unwrap();
		subscriptions.push(SubscriptionTx::Once(SubscriptionOnceTx {
			filter,
			deadline,
			result: None,
		}));
		todo!();
	}

	fn subscribe_frames(&self, filter: CanFilter) -> Subscription {
		todo!();
	}
}

pub struct Subscription {
	frame_rx: mpsc::Receiver<std::io::Result<CanFrame>>,
}

impl Subscription {
	pub async fn recv(&mut self) -> std::io::Result<CanFrame> {
		self.frame_rx.recv()
			.await
			.ok_or_else(|| std::io::ErrorKind::UnexpectedEof)?
	}

	pub fn blocking_recv(&mut self) -> std::io::Result<CanFrame> {
		self.frame_rx.blocking_recv()
			.ok_or_else(|| std::io::ErrorKind::UnexpectedEof)?
	}
}

struct RouterInner {
	socket: CanSocket,
	subscriptions: Mutex<Vec<SubscriptionTx>>,
}

enum SubscriptionTx {
	Once(SubscriptionOnceTx),
	Many(SubscriptionManyTx),
}


struct SubscriptionOnceTx {
	filter: CanFilter,
	deadline: Instant,
	result: Option<std::io::Result<CanFrame>>,
}

struct SubscriptionManyTx {
	filter: CanFilter,
	deadline: Instant,
	frame_tx: mpsc::Sender<std::io::Result<CanFrame>>,
}
