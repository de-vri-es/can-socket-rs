//! Support for the `sync` command.
use std::num::NonZeroU8;

use can_socket::CanFrame;
use crate::CanOpenSocket;

const SYNC_DEFAULT_COB_ID: u8 = 0x80;

/// Send a SYNC command to the CAN network.
pub(crate) async fn send_sync(
	bus: &mut CanOpenSocket,
	counter: Option<NonZeroU8>,
) -> Result<(), std::io::Error> {
	log::debug!("Sending SYNC");
	let frame = match counter {
		Some(counter) => {
			log::debug!("└─ Counter: {counter}");
			CanFrame::new(SYNC_DEFAULT_COB_ID, [counter.get()])
		},
		None => {
			log::debug!("└─ Counter: no counter");
			CanFrame::new(SYNC_DEFAULT_COB_ID, [])
		}
	};

	bus.socket.send(&frame).await
}
