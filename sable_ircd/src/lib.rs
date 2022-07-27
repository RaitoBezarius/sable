//! IRC client server logic.
//!
//! This crate primarily exists to support the [`ClientServer`] type, which
//! extends the [`sable_network::Server`] logic to implement a client server.
//!
//! # Architecture
//!
//! The client server requires, apart from its own identity details:
//!
//!   * An event log
//!   * The receiving half of the event log's RPC channel
//!   * The receiving half of a [`ListenerCollection`]'s event channel
//!
//! The first two of those are passed through to the [`Server`](sable_network::Server);
//! the latter is used to receive client connections.
//!
//! When the [`run`](ClientServer::run) function is called, it spawns the `Server`'s
//! run task in turn, then begins processing events.
//!
//! Client connection events are mediated through the [`ClientConnection`]'s receive queue,
//! which applies rate limiting to received messages before they are passed on to the
//! server's event loop.
//!
//! Network state tracking is handled by the `Server`'s run task; the `ClientServer`
//! receives a stream of [`NetworkHistoryUpdate`s](sable_network::rpc::NetworkHistoryUpdate)
//! which describe changes to the network state as well as which users are to be notified
//! of those changes. The `ClientServer` then translates them into the IRC client
//! protocol.
//!
//! # Command Handling
//!
//! Basic parsing of client commands is in the [`client_message`] module.
//!
//! Command handler registration is at compile time, via the `inventory` crate, mediated
//! by the `command_handler!` macro. To add a new command, create a module under
//! [`command`] and invoke that macro - see one of the existing handlers for examples.
//!
//! Command handlers run with a read-only view of the network and server state. For simple
//! information retrieval (`whois`, `names` and the like), this isn't an issue and the
//! relevant information can simply be sent. For handlers which need to mutate state,
//! they can call [`self.action`](command::CommandHandler::action) to emit a
//! `CommandAction`. These actions will be processed by the `ClientServer`'s event loop
//! and the relevant state changes applied.
//!
//! The most common `CommandAction` variant will be `StateChange`, which creates a new
//! event in the network event log. The event details must be sent to the event log to
//! fill in origin and dependency information before it is sent back to the `Server` for
//! processing. The command handler will not be able to observe the result of the event
//! application; for commands (such as join, part, etc.) which update the network state,
//! the command should not be echoed back to the originating user until the event has
//! reached the update handler after being applied.
//!
//! # Update handling
//!
//! The `ClientServer` receives a stream of `NetworkHistoryUpdate`s from the `Server`
//! task, which describe every event that needs to be handled by the `ClientServer` as
//! well as the set of users who should be notified of the event.
//!
//! Each history update is also stored in the `Server`'s network history log, and can be
//! accessed for later replay in addition to the real time stream.
//!
//! The two traits [`SendHistoryItem`](messages::send_history::SendHistoryItem) and
//! [`SendRealtimeItem`](messages::send_realtime::SendRealtimeItem) handle translation of
//! history log items into client protocol messages. `SendHistoryItem` is implemented for
//! all update types, and is used when replaying history, as well as for real time updates
//! if `SendRealtimeItem` is not implemented for that update type. `SendRealtimeItem`
//! should be used if data is being sent about the current state of the network which is
//! not included in the history log entry, for example to notify a joining user of the
//! current channel membership.

mod command;
mod capability;
mod dns;
mod messages;
mod utils;
mod errors;
mod throttled_queue;

mod client;
use client::*;

mod client_message;
pub use client_message::*;

mod command_processor;
use command_processor::*;

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

mod isupport;
use isupport::*;

mod movable;

pub mod server;
use server::ClientServer;

pub mod prelude;