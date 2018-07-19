// peer.rs
use super::bytes::{Bytes, BytesMut};
use super::futures::sync::mpsc;
use super::futures::{Future, Poll, Stream, Async, task};
use super::tokio::io::Error;
use super::shared::Shared;
use super::lines::Lines;
    
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

type Rx = mpsc::UnboundedReceiver<Bytes>;

pub struct Peer {
    name: BytesMut,
    lines: Lines,
    state: Arc<Mutex<Shared>>,
    rx: Rx,
    addr: SocketAddr,
}

impl Peer {
    pub fn new(
        name: BytesMut,
        state: Arc<Mutex<Shared>>,
        lines: Lines
    ) -> Peer {
        let addr = lines.socket.peer_addr().unwrap();
        let (tx, rx) = mpsc::unbounded();
        state.lock().unwrap()
            .peers.insert(addr, tx);

        Peer {
            name, lines, state, rx, addr,
        }
    }
}

impl Future for Peer {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        const LINES_PER_TICK: usize = 10;

        for i in 0..LINES_PER_TICK {
            match self.rx.poll().unwrap() {
                Async::Ready(Some(v)) => {
                    self.lines.buffer(&v);
                    if i + 1 == LINES_PER_TICK {
                        task::current().notify();
                    }
                }
                _ => break,
            }
        }
        let _ = self.lines.poll_flush()?;

        while let Async::Ready(line) = self.lines.poll()? {
            println!("Received line ({:?}) : {:?}", self.name, line);

            if let Some(message) = line {
                let mut line = self.name.clone();
                line.extend_from_slice(b": ");
                line.extend_from_slice(&message);
                line.extend_from_slice(b"\r\n");

                let line = line.freeze();

                for (addr, tx) in &self.state.lock().unwrap().peers {
                    if *addr != self.addr {
                        tx.unbounded_send(line.clone()).unwrap();
                    }
                }
            } else {
                return Ok(Async::Ready(()));
            }
        }
        Ok(Async::NotReady)
    }    
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.state.lock().unwrap().peers
            .remove(&self.addr);
    }
}


