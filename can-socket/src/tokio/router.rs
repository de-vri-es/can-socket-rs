use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{oneshot, mpsc};
use tokio::time::Instant;

use crate::{CanFilter, CanFrame};
use super::CanSocket;

#[derive(Clone)]
pub struct Router {
	inner: Arc<RouterInner>,
}

impl Router {
	pub async fn recv(&self, filter: CanFilter, timeout: Duration) -> std::io::Result<CanFrame> {
		self.recv_deadline(filter, Instant::now() + timeout).await
	}

	pub async fn recv_deadline(&self, filter: CanFilter, deadline: Instant) -> std::io::Result<CanFrame> {
		let (frame_tx, frame_rx) = oneshot::channel();
		let mut frame_rx = Some(frame_rx);
		self.inner.subscriptions.lock().unwrap().push(SubscriptionTx::Once(SubscriptionOnceTx {
			filter,
			frame_tx: Some(frame_tx),
		}));
		loop {
			tokio::select! {
				frame = self.inner.socket.recv() => {
					self.inner.route_frame(frame);
				},
				frame = recv(&mut frame_rx) => {
					return frame.ok_or(std::io::ErrorKind::UnexpectedEof)?;
				},
				() = tokio::time::sleep_until(deadline) => {
					return Err(std::io::ErrorKind::TimedOut.into())
				}
			}
		}
	}

	pub fn subscribe(&self, filter: CanFilter, queue_capacity: usize) -> Subscription {
		self.subscribe_internal(filter, None, queue_capacity)
	}

	pub fn subscribe_timeout(&self, filter: CanFilter, timeout: Duration, queue_capacity: usize) -> Subscription {
		self.subscribe_internal(filter, Some(Instant::now() + timeout), queue_capacity)
	}

	pub fn subscribe_deadline(&self, filter: CanFilter, deadline: impl Into<Instant>, queue_capacity: usize) -> Subscription {
		self.subscribe_internal(filter, Some(deadline.into()), queue_capacity)
	}

	fn subscribe_internal(&self, filter: CanFilter, deadline: Option<Instant>, queue_capacity: usize) -> Subscription {
		let mut subscriptions = self.inner.subscriptions.lock().unwrap();
		let (frame_tx, frame_rx) = mpsc::channel(queue_capacity);
		subscriptions.push(SubscriptionTx::Many(SubscriptionManyTx {
			filter,
			frame_tx,
		}));
		Subscription {
			frame_rx,
			deadline,
			inner: self.inner.clone(),
		}
	}

	pub async fn send(&self, frame: CanFrame) -> std::io::Result<()> {
		self.inner.socket.send(&frame).await
	}
}

pub struct Subscription {
	frame_rx: mpsc::Receiver<std::io::Result<CanFrame>>,
	deadline: Option<Instant>,
	inner: Arc<RouterInner>,
}

impl Subscription {
	pub async fn recv(&mut self) -> std::io::Result<Option<CanFrame>> {
		if self.deadline.is_some_and(|deadline| Instant::now() >= deadline) {
			return Ok(None)
		}
		let mut deadline = std::pin::pin!(async {
			match self.deadline {
				Some(x) => tokio::time::sleep_until(x).await,
				None => std::future::pending().await,
			}
		});
		loop {
			tokio::select! {
				frame = self.inner.socket.recv() => {
					self.inner.route_frame(frame);
				},
				frame = self.frame_rx.recv()  => {
					return Ok(Some(frame.ok_or(std::io::ErrorKind::UnexpectedEof)??));
				},
				() = &mut deadline => {
					return Ok(None)
				}
			}
		}
	}

	pub fn cancel(&mut self) {
		self.frame_rx.close()
	}
}

struct RouterInner {
	socket: CanSocket,
	subscriptions: Mutex<Vec<SubscriptionTx>>,
}

impl RouterInner {
	fn route_frame(&self, frame: std::io::Result<CanFrame>) {
		let mut subscriptions = self.subscriptions.lock().unwrap();
		subscriptions.retain_mut(|subscription| {
			// If the receiver is closed, drop the subscription.
			if !subscription.is_open() {
				return false;
			}

			// If the subscription doesn't want the frame, bail early.
			// And don't delete the subscription.
			if !subscription.test(&frame) {
				return true;
			}

			// If the subscription is interested in the frame, try to deliver it.
			// If that fails, delete the subscription.
			let frame = frame.as_ref()
				.map(|frame| *frame)
				.map_err(clone_error);
			match subscription.deliver(frame) {
				Ok(()) => true,
				Err(()) => false,
			}
		})
	}
}

enum SubscriptionTx {
	Once(SubscriptionOnceTx),
	Many(SubscriptionManyTx),
}

impl SubscriptionTx {
	fn test(&self, frame: &Result<CanFrame, std::io::Error>) -> bool {
		match frame {
			Ok(frame) => match self {
				Self::Once(x) => x.filter.test(frame),
				Self::Many(x) => x.filter.test(frame),
			},
			// TODO: always deliver all errors?
			Err(_) => true,
		}
	}

	fn deliver(&mut self, frame: std::io::Result<CanFrame>) -> Result<(), ()> {
		match self {
			Self::Once(x) => {
				let frame_tx = x.frame_tx.take().ok_or(())?;
				frame_tx.send(frame).map_err(|_| ())
			},
			Self::Many(x) => x.frame_tx.try_send(frame).map_err(|_| ()),
		}
	}

	fn is_open(&self) -> bool {
		match self {
			Self::Once(x) => x.frame_tx.as_ref().is_some_and(|x| !x.is_closed()),
			Self::Many(x) => !x.frame_tx.is_closed()
		}
	}
}

struct SubscriptionOnceTx {
	filter: CanFilter,
	frame_tx: Option<oneshot::Sender<std::io::Result<CanFrame>>>,
}

struct SubscriptionManyTx {
	filter: CanFilter,
	frame_tx: mpsc::Sender<std::io::Result<CanFrame>>,
}


async fn recv<T>(rx: &mut Option<oneshot::Receiver<T>>) -> Option<T> {
	struct Future<'a, T> {
		rx: &'a mut Option<oneshot::Receiver<T>>,
	}

	impl<'a, T> std::future::Future for Future<'a, T> {
		type Output = Option<T>;

		fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
			use std::pin::Pin;
			let me = self.get_mut();

			let rx = match me.rx {
				None => return std::task::Poll::Ready(None),
				Some(rx) => rx,
			};

			let frame = std::task::ready!(Pin::new(rx).poll(cx)).ok();
			std::task::Poll::Ready(frame)
		}
	}

	Future { rx }.await
}

/// Clone an [`std::io::Error`] with best effor.
///
/// Errors constructed from a raw OS error or an `std::io::ErrorKind` will be cheaply cloned.
///
/// Other errors will be stringified and reconstructed with the same error kind.
fn clone_error(error: &std::io::Error) -> std::io::Error {
	if let Some(raw) = error.raw_os_error() {
		std::io::Error::from_raw_os_error(raw)
	} else if let Some(wrapped) = error.get_ref() {
		std::io::Error::new(error.kind(), wrapped.to_string())
	} else {
		error.kind().into()
	}
}
