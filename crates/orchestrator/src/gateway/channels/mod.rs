//! Canaux messaging Phase 8–10.

pub mod catalog;
mod discord;
mod slack;
mod stub;
mod telegram;
mod webchat;
mod webhook;

pub use catalog::{ChannelCatalog, ChannelDescriptor, CHANNEL_DESCRIPTORS, default_token_env};
pub use discord::{discord_channel, DiscordDelivery};
pub use slack::{slack_channel, SlackDelivery};
pub use stub::stub_channel;
pub use telegram::{telegram_channel, TelegramDelivery};
pub use webchat::webchat_channel;
pub use webhook::{webhook_channel, WebhookChannel, WebhookPayload};