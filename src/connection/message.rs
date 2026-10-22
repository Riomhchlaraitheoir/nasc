//! Defines the RPC [`Message`] type which represents any message sent on the RPC connection

use crate::connection::MethodID;
use derive_more::{Display, From, Into, TryFrom, TryFromReprError};
use pastey::paste;
use std::str::Utf8Error;
use thiserror::Error;

/// Identifies a certain service
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, From, Into, Display, Ord, PartialOrd)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub struct ServiceID(u32);

/// Identifies a certain stream
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, From, Into, Display, Ord, PartialOrd)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub struct StreamID(u32);

impl ServiceID {
    /// Returns true if this service is served by the same peer as the root service
    #[must_use]
    pub const fn is_server_side(self) -> bool {
        self.0.is_multiple_of(2)
    }
}

/// Identifies a certain request
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, From, Display, Ord, PartialOrd)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub struct RequestID(u32);

/// An RPC message
///
/// All message on the connection will be of this type
///
/// For binary serialisation see [`length`](Self::length), [`serialise`](Self::serialise),
/// [`serialise_into`](Self::serialise_into) and [`deserialise`](Self::deserialise)
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub enum Message {
    /// Represents a request to a service/method
    Request {
        /// The ID for the target service
        service_id: ServiceID,
        /// The ID for this request, used to identify response
        request_id: RequestID,
        /// The ID for the target method
        method_id: MethodID,
        /// The method arguments
        payload: Vec<u8>,
    },
    /// Cancel a particular request if possible, the response may still be sent due to race conditions
    CancelRequest {
        /// The request to cancel
        request_id: RequestID,
    },
    /// Represents a response from a method
    Response {
        /// The request ID, corresponds to `request_id` in [`Request`](Self::Request)
        request_id: RequestID,
        /// The result of the call
        ///
        /// - [`Ok`]: the method called successfully and this is the return value
        /// - [`Err`]: The method could not be called, this is the relevant error
        result: Result<Vec<u8>, HandleError>,
    },
    /// A stream-related message
    Stream {
        /// The ID of the stream
        stream_id: StreamID,
        /// The stream messge
        message: StreamMessage,
    },
    /// The client for this service was dropped, the correspond handler can also be dropped
    ServiceDropped {
        /// The identity of the dropped service
        service_id: ServiceID,
    },
    /// An error occurred on the connection, separate from any identifiable request and the connection should be closed
    ConnectionError(ConnectionError),
}

/// A Stream-related message for a particular stream
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub enum StreamMessage {
    /// Poll the stream
    Poll,
    /// Item from stream/stream exhausted
    Item(Option<Vec<u8>>),
    /// Stream client-side has been dropped
    Dropped,
}

/// Indicates that the transport was successful, but the response indicated some problem
#[derive(Debug, Eq, PartialEq, Clone, Error)]
#[non_exhaustive]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
pub enum HandleError {
    /// No service found by the selected ID
    #[error("Service not found")]
    ServiceNotFound,
    /// No method found by the selected ID
    #[error("Method not found")]
    MethodNotFound,
    /// Request was improperly formatted
    #[error("Request was rejected: {0}")]
    BadPayloadFormat(String),
    /// Request timed out
    #[error("Request timed out")]
    Timeout,
    /// Internal Server Error
    #[error("Internal Server Error: {0}")]
    InternalError(String),
    /// Unknown error type
    #[cfg_attr(all(test, not(target_arch="wasm32")), proptest(skip))]
    #[error("Unknown error type")]
    UnknownErrorType(u8, String),
}

/// An error on a connection which is not related to any particular request
#[derive(Error, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
#[non_exhaustive]
pub enum ConnectionError {
    /// The frame structure itself is corrupted or violates the protocol rules.
    /// This includes unknown tags or unparseable headers.
    #[error("Received an invalid or malformed protocol frame: {0}")]
    MessageMalformed(String),

    /// A fallback variant for other general internal errors
    #[error("Internal error: {0}")]
    InternalError(String),
    /// Unknown error type
    #[cfg_attr(all(test, not(target_arch="wasm32")), proptest(skip))]
    #[error("Unknown error type")]
    UnknownErrorType(u8, String),
}

#[derive(TryFrom)]
#[try_from(repr)]
#[repr(u8)]
enum MessageTagRaw {
    Request = 0,
    CancelRequest,
    Response,
    ServiceDropped,
    StreamPoll,
    StreamItem,
    StreamDropped,
    StreamDone,

    HandleErrorServiceNotFound = 128,
    HandleErrorMethodNotFound,
    HandleErrorBadPayloadFormat,
    HandleErrorTimeout,
    HandleErrorInternalError,

    ConnectionErrorMessageMalformed = 192,
    ConnectionErrorInternalError,
}

enum MessageTag {
    Request,
    CancelRequest,
    Response,
    ServiceDropped,
    Stream(StreamMessageTag),
    HandleError(HandleErrorTag),
    ConnectionError(ConnectionErrorTag),
    Unknown(u8),
}

enum StreamMessageTag {
    Poll,
    Item,
    Dropped,
    Done,
}

enum HandleErrorTag {
    ServiceNotFound,
    MethodNotFound,
    BadPayloadFormat,
    InternalError,
    Timeout,
    Unknown(u8),
}

enum ConnectionErrorTag {
    MessageMalformed,
    InternalError,
    Unknown(u8),
}

impl From<Result<MessageTagRaw, TryFromReprError<u8>>> for MessageTag {
    fn from(result: Result<MessageTagRaw, TryFromReprError<u8>>) -> Self {
        let result = result.map_err(|e| e.input);
        match result {
            Ok(MessageTagRaw::Request) => Self::Request,
            Ok(MessageTagRaw::CancelRequest) => Self::CancelRequest,
            Ok(MessageTagRaw::Response) => Self::Response,
            Ok(MessageTagRaw::ServiceDropped) => Self::ServiceDropped,
            Ok(MessageTagRaw::StreamPoll) => Self::Stream(StreamMessageTag::Poll),
            Ok(MessageTagRaw::StreamItem) => Self::Stream(StreamMessageTag::Item),
            Ok(MessageTagRaw::StreamDropped) => Self::Stream(StreamMessageTag::Dropped),
            Ok(MessageTagRaw::StreamDone) => Self::Stream(StreamMessageTag::Done),
            Err(tag @ 0..128) => Self::Unknown(tag),

            Ok(MessageTagRaw::HandleErrorServiceNotFound) => {
                Self::HandleError(HandleErrorTag::ServiceNotFound)
            }
            Ok(MessageTagRaw::HandleErrorMethodNotFound) => {
                Self::HandleError(HandleErrorTag::MethodNotFound)
            }
            Ok(MessageTagRaw::HandleErrorBadPayloadFormat) => {
                Self::HandleError(HandleErrorTag::BadPayloadFormat)
            }
            Ok(MessageTagRaw::HandleErrorInternalError) => {
                Self::HandleError(HandleErrorTag::InternalError)
            }
            Ok(MessageTagRaw::HandleErrorTimeout) => {
                Self::HandleError(HandleErrorTag::Timeout)
            }
            Err(tag @ 128..192) => Self::HandleError(HandleErrorTag::Unknown(tag)),

            Ok(MessageTagRaw::ConnectionErrorMessageMalformed) => {
                Self::ConnectionError(ConnectionErrorTag::MessageMalformed)
            }
            Ok(MessageTagRaw::ConnectionErrorInternalError) => {
                Self::ConnectionError(ConnectionErrorTag::InternalError)
            }
            Err(tag @ 192..=255) => Self::ConnectionError(ConnectionErrorTag::Unknown(tag)),
        }
    }
}

#[allow(clippy::fallible_impl_from, reason = "unexpected case is only possible if there is a seriously weird bug where this lib explicitly creates a unknown tag value")]
impl From<MessageTag> for MessageTagRaw {
    fn from(tag: MessageTag) -> Self {
        match tag {
            MessageTag::Request => Self::Request,
            MessageTag::CancelRequest => Self::CancelRequest,
            MessageTag::Response => Self::Response,
            MessageTag::ServiceDropped => Self::ServiceDropped,
            MessageTag::HandleError(HandleErrorTag::ServiceNotFound) => {
                Self::HandleErrorServiceNotFound
            }
            MessageTag::HandleError(HandleErrorTag::MethodNotFound) => {
                Self::HandleErrorMethodNotFound
            }
            MessageTag::HandleError(HandleErrorTag::BadPayloadFormat) => {
                Self::HandleErrorBadPayloadFormat
            }
            MessageTag::HandleError(HandleErrorTag::InternalError) => {
                Self::HandleErrorInternalError
            }
            MessageTag::HandleError(HandleErrorTag::Timeout) => {
                Self::HandleErrorTimeout
            }
            MessageTag::ConnectionError(ConnectionErrorTag::MessageMalformed) => {
                Self::ConnectionErrorMessageMalformed
            }
            MessageTag::ConnectionError(ConnectionErrorTag::InternalError) => {
                Self::ConnectionErrorInternalError
            }
            MessageTag::Stream(StreamMessageTag::Poll) => Self::StreamPoll,
            MessageTag::Stream(StreamMessageTag::Item) => Self::StreamItem,
            MessageTag::Stream(StreamMessageTag::Dropped) => Self::StreamDropped,
            MessageTag::Stream(StreamMessageTag::Done) => Self::StreamDone,
            MessageTag::Unknown(tag)
            | MessageTag::HandleError(HandleErrorTag::Unknown(tag))
            | MessageTag::ConnectionError(ConnectionErrorTag::Unknown(tag)) => {
                panic!("Cannot write unknown tag {tag} to connection")
            }
        }
    }
}

impl Message {
    /// The number of bytes required by this message once serialised
    #[must_use]
    pub const fn length(&self) -> usize {
        size_of::<u8>()
            + match self {
                Self::Request {
                    service_id: _,
                    request_id: _,
                    method_id: _,
                    payload,
                } => {
                    size_of::<ServiceID>()
                        + size_of::<RequestID>()
                        + size_of::<MethodID>()
                        + size_of::<u32>()
                        + payload.len()
                }
                Self::CancelRequest { request_id: _ } => size_of::<RequestID>(),
                Self::Response {
                    request_id: _,
                    result,
                } => {
                    size_of::<RequestID>()
                        + size_of::<u32>()
                        + match result {
                            Ok(payload) => payload.len(),
                            Err(error) => error.tag_and_message().1.len(),
                        }
                }
                Self::ServiceDropped { service_id: _ } => size_of::<ServiceID>(),
                Self::ConnectionError(error) => size_of::<u32>() + error.tag_and_message().1.len(),
                Self::Stream {
                    stream_id: _,
                    message,
                } => size_of::<StreamID>() + if let StreamMessage::Item(Some(payload)) = message {
                    size_of::<u32>() + payload.len()
                } else {
                    0
                },
            }
    }

    /// Serialise this message into a new Vec
    ///
    /// To use an existing Vec, see [`serialise_into`](Self::serialise_into)
    #[must_use]
    pub fn serialise(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.length());
        self.serialise_into(&mut buf);
        buf
    }

    /// Serialise into an existing buffer
    pub fn serialise_into(&self, buf: &mut Vec<u8>) {
        let mut buf = WriteBuffer { buffer: buf };
        match self {
            Self::Request {
                service_id,
                request_id,
                method_id,
                payload,
            } => {
                buf.write_tag(MessageTag::Request);
                buf.write_u32(service_id.0);
                buf.write_u32(request_id.0);
                buf.write_u32(method_id.0);
                buf.write_bytes(payload);
            }
            Self::CancelRequest {
                request_id,
            } => {
                buf.write_tag(MessageTag::CancelRequest);
                buf.write_u32(request_id.0);
            }
            Self::Response {
                request_id,
                result: Ok(payload),
            } => {
                buf.write_tag(MessageTag::Response);
                buf.write_u32(request_id.0);
                buf.write_bytes(payload);
            }
            Self::ServiceDropped { service_id } => {
                buf.write_tag(MessageTag::ServiceDropped);
                buf.write_u32(service_id.0);
            }
            Self::Response {
                request_id,
                result: Err(error),
            } => {
                let (tag, message) = match error {
                    HandleError::ServiceNotFound => (HandleErrorTag::ServiceNotFound, ""),
                    HandleError::MethodNotFound => (HandleErrorTag::MethodNotFound, ""),
                    HandleError::BadPayloadFormat(message) => {
                        (HandleErrorTag::BadPayloadFormat, message.as_str())
                    }
                    HandleError::InternalError(message) => {
                        (HandleErrorTag::InternalError, message.as_str())
                    }
                    HandleError::UnknownErrorType(_, _) => {
                        panic!("UnknownErrorType should never be written to connection")
                    }
                    HandleError::Timeout => (HandleErrorTag::Timeout, "")
                };
                buf.write_tag(MessageTag::HandleError(tag));
                buf.write_u32(request_id.0);
                buf.write_str(message);
            }
            Self::ConnectionError(error) => {
                let (tag, message) = match error {
                    ConnectionError::MessageMalformed(message) => {
                        (ConnectionErrorTag::MessageMalformed, message.as_str())
                    }
                    ConnectionError::InternalError(message) => {
                        (ConnectionErrorTag::InternalError, message.as_str())
                    }
                    ConnectionError::UnknownErrorType(_, _) => {
                        panic!("UnknownErrorType should never be written to connection")
                    }
                };
                buf.write_tag(MessageTag::ConnectionError(tag));
                buf.write_str(message);
            }
            Self::Stream {
                stream_id,
                message: msg,
            } => {
                let tag = match msg {
                    StreamMessage::Poll => StreamMessageTag::Poll,
                    StreamMessage::Item(Some(_)) => StreamMessageTag::Item,
                    StreamMessage::Dropped => StreamMessageTag::Dropped,
                    StreamMessage::Item(None) => StreamMessageTag::Done,
                };
                buf.write_tag(MessageTag::Stream(tag));
                buf.write_u32(stream_id.0);
                match msg {
                    StreamMessage::Poll | StreamMessage::Dropped | StreamMessage::Item(None) => {}
                    StreamMessage::Item(Some(item)) => buf.write_bytes(item),
                }
            }
        }
    }

    /// Deserialise a message from bytes
    ///
    /// # Errors
    /// Returns an error if parsing fails
    pub fn deserialise(buf: &[u8]) -> Result<Self, MessageParseError> {
        if buf.is_empty() {
            return Err(MessageParseError::UnexpectedEOF);
        }
        let mut buf = ReadBuffer {
            buffer: buf,
            read_len: 0,
        };
        let tag = buf.read_tag();
        Ok(match tag {
            MessageTag::Request => {
                buf.check_bounds(
                    size_of::<ServiceID>()
                        + size_of::<RequestID>()
                        + size_of::<MethodID>()
                        + size_of::<u32>(),
                )?;
                let service_id: ServiceID = buf.read_u32().into();
                let request_id: RequestID = buf.read_u32().into();
                let method_id: MethodID = MethodID(buf.read_u32());
                let payload = buf.read_bytes()?.to_vec();
                Self::Request {
                    service_id,
                    request_id,
                    method_id,
                    payload,
                }
            }
            MessageTag::CancelRequest => {
                buf.check_bounds(size_of::<RequestID>())?;
                let request_id: RequestID = buf.read_u32().into();
                Self::CancelRequest { request_id }
            }
            MessageTag::Response => {
                buf.check_bounds(size_of::<RequestID>() + size_of::<u32>())?;
                let request_id: RequestID = buf.read_u32().into();
                let payload = buf.read_bytes()?.to_vec();
                Self::Response {
                    request_id,
                    result: Ok(payload),
                }
            }
            MessageTag::HandleError(tag) => {
                buf.check_bounds(size_of::<RequestID>())?;
                let request_id: RequestID = buf.read_u32().into();
                let message = buf.read_string()?;
                let error = match tag {
                    HandleErrorTag::ServiceNotFound => HandleError::ServiceNotFound,
                    HandleErrorTag::MethodNotFound => HandleError::MethodNotFound,
                    HandleErrorTag::BadPayloadFormat => {
                        HandleError::BadPayloadFormat(message.to_owned())
                    }
                    HandleErrorTag::InternalError => HandleError::InternalError(message.to_owned()),
                    HandleErrorTag::Timeout => HandleError::Timeout,
                    HandleErrorTag::Unknown(tag) => HandleError::UnknownErrorType(tag, message.to_owned()),
                };
                Self::Response {
                    request_id,
                    result: Err(error),
                }
            }
            MessageTag::ServiceDropped => {
                buf.check_bounds(size_of::<ServiceID>())?;
                let service_id: ServiceID = buf.read_u32().into();
                Self::ServiceDropped { service_id }
            }
            MessageTag::ConnectionError(tag) => {
                let message = buf.read_string()?;
                let error = match tag {
                    ConnectionErrorTag::MessageMalformed => {
                        ConnectionError::MessageMalformed(message.to_owned())
                    }
                    ConnectionErrorTag::InternalError => {
                        ConnectionError::InternalError(message.to_owned())
                    }
                    ConnectionErrorTag::Unknown(tag) => ConnectionError::UnknownErrorType(tag, message.to_owned()),
                };
                Self::ConnectionError(error)
            }
            MessageTag::Stream(tag) => {
                let stream_id: StreamID = buf.read_u32().into();
                let msg = match tag {
                    StreamMessageTag::Poll => StreamMessage::Poll,
                    StreamMessageTag::Item => {
                        let item = buf.read_bytes()?;
                        StreamMessage::Item(Some(item.to_vec()))
                    }
                    StreamMessageTag::Dropped => StreamMessage::Dropped,
                    StreamMessageTag::Done => StreamMessage::Item(None),
                };
                Self::Stream {
                    stream_id,
                    message: msg,
                }
            }
            MessageTag::Unknown(tag) => return Err(MessageParseError::UnknownTag(tag)),
        })
    }
}

struct WriteBuffer<'a> {
    buffer: &'a mut Vec<u8>,
}

impl WriteBuffer<'_> {
    fn write_u8(&mut self, n: u8) {
        self.buffer.push(n);
    }
    fn write_u32(&mut self, n: u32) {
        self.buffer.extend_from_slice(&n.to_le_bytes());
    }

    fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }
    fn write_bytes(&mut self, bytes: &[u8]) {
        let Ok(len) = bytes.len().try_into() else {
            panic!("Cannot write more than 2^32 bytes");
        };
        self.write_u32(len);
        self.buffer.extend_from_slice(bytes);
    }

    fn write_tag(&mut self, tag: MessageTag) {
        let raw: MessageTagRaw = tag.into();
        self.write_u8(raw as u8);
    }
}

struct ReadBuffer<'a> {
    buffer: &'a [u8],
    read_len: usize,
}

macro_rules! ints {
    ($($u:ident),*) => {
        paste! {$(
    fn [<read_ $u>](&mut self) -> $u {
        let value = $u::from_le_bytes(self.buffer[self.read_len..self.read_len + size_of::<$u>()].try_into().unwrap());
        self.read_len += size_of::<$u>();
        value
    })*}
    };
}

impl<'a> ReadBuffer<'a> {
    const fn check_bounds(&self, size: usize) -> Result<(), MessageParseError> {
        if self.buffer.len() < self.read_len + size {
            Err(MessageParseError::UnexpectedEOF)
        } else {
            Ok(())
        }
    }

    fn read_bytes(&mut self) -> Result<&'a [u8], MessageParseError> {
        self.check_bounds(size_of::<u32>())?;
        let size = self.read_u32() as usize;
        self.check_bounds(size)?;
        let start = self.read_len;
        let end = start + size;
        self.read_len = end;
        Ok(&self.buffer[start..end])
    }

    ints!(u8, u32);

    fn read_string(&mut self) -> Result<&'a str, MessageParseError> {
        Ok(str::from_utf8(self.read_bytes()?)?)
    }

    fn read_tag(&mut self) -> MessageTag {
        let raw: Result<MessageTagRaw, _> = self.read_u8().try_into();
        raw.into()
    }
}

impl HandleError {
    const fn tag_and_message(&self) -> (HandleErrorTag, &str) {
        match self {
            Self::ServiceNotFound => (HandleErrorTag::ServiceNotFound, ""),
            Self::MethodNotFound => (HandleErrorTag::MethodNotFound, ""),
            Self::BadPayloadFormat(message) => {
                (HandleErrorTag::BadPayloadFormat, message.as_str())
            }
            Self::InternalError(message) => {
                (HandleErrorTag::InternalError, message.as_str())
            }
            Self::Timeout => {
                (HandleErrorTag::Timeout, "")
            }
            Self::UnknownErrorType(_, _) => {
                panic!("UnknownErrorType should never be written to connection")
            }
        }
    }
}

impl ConnectionError {
    const fn tag_and_message(&self) -> (ConnectionErrorTag, &str) {
        match self {
            Self::MessageMalformed(message) => {
                (ConnectionErrorTag::MessageMalformed, message.as_str())
            }
            Self::InternalError(message) => {
                (ConnectionErrorTag::InternalError, message.as_str())
            }
            Self::UnknownErrorType(_, _) => {
                panic!("UnknownErrorType should never be written to connection")
            }
        }
    }
}

/// An error in parsing a [`Message`]
#[derive(Debug, Error)]
pub enum MessageParseError {
    /// Unexpected end of input
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    /// Unknown message tag
    #[error("Unexpected tag: {0}")]
    UnknownTag(u8),
    /// UTF-8 decode failed
    #[error("UTF-8 decode error: {0}")]
    UTF8(#[from] Utf8Error),
}

impl From<MessageParseError> for ConnectionError {
    fn from(error: MessageParseError) -> Self {
        Self::MessageMalformed(error.to_string())
    }
}

impl From<MessageParseError> for Message {
    fn from(error: MessageParseError) -> Self {
        Self::ConnectionError(error.into())
    }
}


#[cfg(all(test, not(target_arch="wasm32")))]
mod tests {
    use proptest::arbitrary::any;
    use proptest::prelude::ProptestConfig;
    use proptest::proptest;
    use super::*;

    const SAMPLE_MESSAGES: &[Message] = &[
        Message::Request {
            service_id: ServiceID(0),
            request_id: RequestID(34),
            method_id: MethodID(100),
            payload: vec![],
        },
        Message::Stream {
            stream_id: StreamID(0),
            message: StreamMessage::Poll,
        }
    ];

    #[test]
    fn test_serialisation_known_cases() {
        let mut buffer = Vec::new();
        for message in SAMPLE_MESSAGES {
            message.serialise_into(&mut buffer);
            assert_eq!(buffer.len(), message.length());
            let deserialised = Message::deserialise(&buffer);
            assert_eq!(message, &deserialised.expect("Failed to deserialise"));
            buffer.clear();
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn test_serialisation_fuzz(message in any::<Message>()) {
            let mut buffer = Vec::new();
            message.serialise_into(&mut buffer);
            assert_eq!(buffer.len(), message.length());
            let deserialised = Message::deserialise(&buffer);
            assert_eq!(message, deserialised.expect("Failed to deserialise"));
            buffer.clear();
        }
    }

    // #[test]
    // fn debug_message() {
    //     let mut buffer = include_bytes!("../../debug.bin");
    //     let message = Message::deserialise(buffer).expect("Failed to deserialise");
    //     println!("{message:?}");
    // }
}