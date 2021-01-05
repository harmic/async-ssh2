use crate::Error;
use async_io::Async;
use std::{io, net::TcpStream};
use futures::future;
use futures_util;

pub async fn run_ssh2_fn<R, F: FnMut() -> Result<R, ssh2::Error>>(
    stream: &Async<TcpStream>,
    mut cb: F,
) -> Result<R, Error> {

    loop {
        match cb() {
            Ok(v) => return Ok(v),
            Err(e) if io::Error::from(ssh2::Error::from_errno(e.code())).kind()
            == io::ErrorKind::WouldBlock => (),
            Err(e) => return Err(Error::from(e))
        }
        // Wait until the I/O handle is readable or writable.
        let readable = stream.readable();
        let writable = stream.writable();
        futures_util::pin_mut!(readable);
        futures_util::pin_mut!(writable);
        let _ = future::select(readable, writable).await;        
    }

    /*
    let res = stream
        .read_with(|_s| match cb() {
            Ok(v) => Ok(Ok(v)),
            Err(e)
                if io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                    == io::ErrorKind::WouldBlock =>
            {
                Err(io::Error::new(io::ErrorKind::WouldBlock, e))
            }
            Err(e) => Ok(Err(e)),
        })
        .await??;
    Ok(res)*/
}
