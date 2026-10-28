//! Defines types for using a channel pair as a connection, either for testing or to allow more
//! flexibility with managing the actual connection

use std::fmt::Display;
use futures::FutureExt;
use std::pin::pin;
use crate::connection::message::Message;
use crate::connection::{Connection, ConnectionDriver};
use async_channel::{Receiver, Sender, bounded};
use futures::select;
use tracing::error;

/// A channel-base connection, see [Module level documentation](self)
pub struct ChannelConnection {
    incoming: Receiver<Message>,
    outgoing: Sender<Message>,
}

impl ChannelConnection {
    /// Create a new connection from the given channels
    #[must_use]
    pub const fn new(incoming: Receiver<Message>, outgoing: Sender<Message>) -> Self {
        Self { incoming, outgoing }
    }

    /// Create a new connection and return channels to be used to drive the connection
    #[must_use]
    pub fn pair() -> (Self, Sender<Message>, Receiver<Message>) {
        let incoming = bounded(10);
        let outgoing = bounded(10);
        let conn = Self {
            incoming: incoming.1,
            outgoing: outgoing.0,
        };
        (conn, incoming.0, outgoing.1)
    }
}

impl Connection for ChannelConnection {
    async fn run(
        self,
        driver: ConnectionDriver,
    ) -> Result<(), impl Display> {
        let mut next_message_in = pin!(self.incoming.recv().fuse());
        let mut next_message_out = pin!(driver.next_outgoing_message().fuse());
        let mut teardown_signal = pin!(driver.teardown().fuse());
        loop {
            select! {
                complete => return Ok(()),
                () = teardown_signal => {
                    for message in driver.flush_outgoing_messages() {
                        if let Err(error) = self.outgoing.send(message).await {
                            return Err(format!("Send Error: {error}"));
                        }
                    }
                    return Ok(());
                }
                result = next_message_in => {
                    next_message_in.set(self.incoming.recv().fuse());
                    if let Ok(message) = result {
                        if let Err(error) = driver.consume_incoming_message(message).await {
                            return Err(format!("Connection Error: {error}"));
                        }
                    } else {
                        return Ok(());
                    }
                },
                option = &mut next_message_out => {
                    next_message_out.set(driver.next_outgoing_message().fuse());
                    let Some(message) = option else {
                        error!("Responses channel unexpectedly closed");
                        return Ok(())
                    };
                    if let Err(error) = self.outgoing.send(message).await {
                        error!("Connection closed");
                        return Err(format!("Connection Error: {error}"));
                    }
                }
            }
        }
    }
}