// shared.rs

use super::futures::sync::mpsc;
use super::bytes::{Bytes};
use std::collections::HashMap;
use std::net::SocketAddr;

type Tx = mpsc::UnboundedSender<Bytes>;

pub struct Shared {
    pub peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    pub fn new() -> Self {
        return Shared {
            peers: HashMap::new(),
        }
    }
}
