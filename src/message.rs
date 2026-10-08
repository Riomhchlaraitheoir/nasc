//! Defines message types

use crate::client::HandleError;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub(crate) enum ConnectionMessage<T> {
    ConnectionError(ConnectionError),
    Payload {
        request_id: RequestID,
        payload: T,
    },
    HandleError {
        request_id: RequestID,
        error: HandleError,
    },
    StreamItem {
        stream_id: StreamID,
        item: Option<Vec<u8>>
    },
    StreamPoll {
        stream_id: StreamID,
    }
}

/// Identifies a certain request
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, From, Display, Ord, PartialOrd)]
pub struct RequestID(u32);

/// Identifies a certain stream
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, From, Display, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StreamID(u32);

/// An error on a connection which is not related to any particular request
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ConnectionError {
    /// Internal error: the response channel has closed when it should have remained open
    #[error("Response channel closed unexpectedly")]
    ResponseChannelClosed,
    #[error("Message packet improperly formatted")]
    /// This indicates that a malformed message error was received from the connection.
    /// This should never be sent back into the connection since that could cause an infinite
    /// loop with errors being sent back and forth on the connection.
    /// This will never be read from a connections
    MalformedMessageError,
    /// This indicates that a message was received with invalid formatting
    #[error("Message packet improperly formatted")]
    MessageWasMalformed
}

#[allow(deprecated)]
impl ConnectionMessage<Vec<u8>> {
    pub fn format(self) -> Vec<u8> {
        let mut message = Vec::new();
        match self {
            Self::ConnectionError(error) => {
                message.push(1);
                match error {
                    ConnectionError::ResponseChannelClosed => {
                        message.push(0);
                    }
                    ConnectionError::MessageWasMalformed => {
                        message.push(1);
                    }
                    ConnectionError::MalformedMessageError => {
                        message.push(2);
                    }
                }
            }
            Self::Payload { request_id, payload } => {
                message.push(0);
                message.extend(request_id.0.to_le_bytes());
                message.extend(&*payload);
            }
            Self::HandleError { request_id, error } => {
                message.push(2);
                message.extend(request_id.0.to_le_bytes());
                match error {
                    HandleError::BadRequest(error) => {
                        message.push(0);
                        message.extend(error.into_bytes());
                    }
                    HandleError::InternalServerError(error) => {
                        message.push(1);
                        message.extend(error.into_bytes());
                    }
                    HandleError::Unexpected => {
                        message.push(2);
                    }
                }
            }
            Self::StreamItem { stream_id, item } => {
                message.push(3);
                message.extend(stream_id.0.to_le_bytes());
                if let Some(item) = item {
                    message.extend(item);
                }
            }
            Self::StreamPoll { stream_id } => {
                message.push(4);
                message.extend(stream_id.0.to_le_bytes());
            }
        }
        message
    }

    /// Get request id and payload from the given request/response. Useful for implementing transport
    /// protocols that share a single connection for many concurrent requests
    pub fn parse(message: &[u8]) -> Self {
        let Some(&message_type) = message.first() else {
            return Self::ConnectionError(ConnectionError::MessageWasMalformed);
        };
        let message = &message[1..];
        if message_type == 1 {
            let error_type = message[0];
            return match error_type {
                0 => Self::ConnectionError(ConnectionError::ResponseChannelClosed),
                1 => Self::ConnectionError(ConnectionError::MalformedMessageError),
                _ => Self::ConnectionError(ConnectionError::MessageWasMalformed),
            }
        }
        if message.len() < 4 {
            return Self::ConnectionError(ConnectionError::MessageWasMalformed);
        }
        match message_type {
            0 => {
                let request_id = u32::from_le_bytes([message[0], message[1], message[2], message[3]]).into();
                let message = &message[4..];
                Self::Payload {
                    request_id,
                    payload: message.to_vec(),
                }
            },
            2 => {
                let request_id = u32::from_le_bytes([message[0], message[1], message[2], message[3]]).into();
                let message = &message[4..];
                let error_type = &message[0];
                let message = &message[1..];
                let error = match *error_type {
                    0 | 1 => {
                        let error = String::from_utf8_lossy(message);
                        match error_type {
                            0 => HandleError::BadRequest(error.into_owned()),
                            1 => HandleError::InternalServerError(error.into_owned()),
                            _ => unreachable!(),
                        }
                    }
                    2 => HandleError::Unexpected,
                    _ => {
                        return Self::ConnectionError(ConnectionError::MessageWasMalformed);
                    }
                };
                Self::HandleError { request_id, error }
            }
            3 => {
                let stream_id: StreamID = u32::from_le_bytes([message[0], message[1], message[2], message[3]]).into();
                let item = (message.len() >= 4).then(||{
                    message[4..].to_vec()
                });
                Self::StreamItem {
                    stream_id, item
                }
            },
            4 => {
                let stream_id: StreamID = u32::from_le_bytes([message[0], message[1], message[2], message[3]]).into();
                Self::StreamPoll { stream_id }
            },
            _ => Self::ConnectionError(ConnectionError::MessageWasMalformed),
        }
    }
}
