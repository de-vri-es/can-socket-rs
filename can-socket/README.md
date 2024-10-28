CAN socket

This library exposes a [`CanSocket`] and related types,
allowing you to communicate over a Controller Area Network (CAN) bus.

The is a standard blocking or non-blocking [`CanSocket`],
and an asynchronous [`tokio::CanSocket`].

This library uses the `SocketCAN` interface and only works on Linux.

Supported features:
* Bind sockets to specific interfaces by name or index.
* Bind sockets to *all* CAN interfaces at the same time.
* Send and receive data frames and RTR frames.
* Send and receive standard frames and extended frames.
* Setting per-socket filters.
* Control over the `loopback` and `recv_own_msgs` options.
* Constructing compile-time checked CAN IDs.

[`CanSocket`]: https://docs.rs/can-socket/latest/can_socket/struct.CanSocket.html
[`tokio::CanSocket`]: https://docs.rs/can-socket/latest/can_socket/tokio/struct.CanSocket.html
