//! Messagerie inter-agents (inbox / outbox).

mod message;
mod receive;
mod send;

pub use message::AgentMessage;
pub use receive::receive_messages;
pub use send::send_message;