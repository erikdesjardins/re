use crate::config::COPY_BUFFER_SIZE;
use std::ops::RangeInclusive;
use std::time::Duration;

pub const QUEUE_TIMEOUT: Duration = Duration::from_secs(60);
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

pub const SERVER_ACCEPT_BACKOFF_SECS: RangeInclusive<u8> = 1..=64;
pub const CLIENT_BACKOFF_SECS: RangeInclusive<u8> = 1..=64;

pub const WS_MAX_MESSAGE_SIZE: usize = COPY_BUFFER_SIZE;
