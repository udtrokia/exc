// lines.rs
use super::bytes::{BytesMut, BufMut};
use super::tokio::net::TcpStream;
use super::tokio::io::*;
use super::tokio::prelude::*;
use super::futures::Stream;
use super::futures::Async;

pub struct Lines {
    pub socket: TcpStream,
    rd: BytesMut,
    wr: BytesMut,
}

impl Lines {
    pub fn new(socket: TcpStream) -> Self {
        Lines {
            socket,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
        }
    }

    pub fn buffer(&mut self, line: &[u8]) {
        self.wr.reserve(line.len());
        self.wr.put(line);
    }

    pub fn poll_flush(&mut self) -> Poll<(), Error> {
        while !self.wr.is_empty() {
            let n = try_ready!(self.socket.poll_write(&self.wr));
            assert!(n > 0);
            let _ = self.wr.split_to(n);            
        }
        Ok(Async::Ready(()))
    }

    pub fn fill_read_buf(&mut self) -> Poll<(), Error> {
        loop {
            self.rd.reserve(1024);
            let n = try_ready!(self.socket.read_buf(&mut self.rd));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}

impl Stream for Lines {
    type Item = BytesMut;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let sock_closed = self.fill_read_buf()?.is_ready();

        let pos = self.rd.windows(2).enumerate()
            .find(|&(_, bytes)| bytes == b"\r\n")
            .map(|(i, _)| i);

        if let Some(pos) = pos {
            let mut line = self.rd.split_to(pos + 2);
            line.split_off(pos);
            return Ok(Async::Ready(Some(line)));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}
