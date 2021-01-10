use crate::{channel::Channel, Error};
use async_io::{Async, Timer};
use ssh2::{self};
use std::{net::TcpStream, sync::Arc, time::Duration};

/// See [`Listener`](ssh2::Listener).
pub struct Listener<'a> {
    inner: ssh2::Listener,
    inner_session: &'a ssh2::Session,
    stream: Arc<Async<TcpStream>>,
}

impl<'a> Listener<'a> {
    pub(crate) fn new<'b>(listener: ssh2::Listener, session: &'b ssh2::Session, stream: Arc<Async<TcpStream>>) -> Listener<'b> {
        Listener {
            inner: listener,
            inner_session: session,
            stream,
        }
    }

    /// See [`accept`](ssh2::Listener::accept).
    pub async fn accept<'b>(&'b mut self) -> Result<Channel<'b>, Error> {
        // The I/O object for Listener::accept is on the remote SSH server. There is no way to poll
        // its state so the best we can do is loop and check whether we have a new connection every
        // 10ms.
        let channel = loop {
            match self.inner.accept() {
                Ok(channel) => break channel,
                Err(e)
                    if std::io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                        == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(Error::SSH2(e)),
            };

            Timer::after(Duration::from_millis(10)).await;
        };

        Ok(Channel::new(channel, self.inner_session, self.stream.clone()))
    }
}
