extern crate exc;

use exc::tokio;
use exc::tokio::net::{TcpListener, TcpStream};
use exc::tokio::prelude::{Stream, Future};
use exc::futures::future;
use exc::futures::future::Either;
use exc::shared::Shared;
use exc::lines::Lines;
use exc::peer::Peer;
use std::sync::{Arc, Mutex};

// This is how the data flush.
fn process(socket: TcpStream, state: Arc<Mutex<Shared>>) {
    // Lines { socket, rd, wd }
    let lines = Lines::new(socket);

    // __into-futures__: consumes this object and produces a future
    // __Future__: Asynchronous values. contains:
    // + Future trait,
    // + FutureExt trait, provide adapters for chaining and composing futures
    // + Top-level future combinators like `lazy` which creates future.
    let connection = lines.into_future()
        .map_err(|(e, _)| e)
        .and_then(|(name, lines)| {
            let name = match name {
                Some(name) => name,
                None => {
                    // enum the value.
                    return Either::A(future::ok(()))
                }
            };
            println!("`{:?}` is joining the space", name);

            // Peer { name, lines, state, rx, socketAddr }
            let peer = Peer::new(name, state, lines);
            Either::B(peer)
        }).map_err(|e| {
            println!("connection error = {:?}", e);
        });
    tokio::spawn(connection);
}

fn main() {
    // The Shared Data. This is how all the peers communicate.
    let state = Arc::new(Mutex::new(Shared::new()));

    // Client Addr
    let addr = "0.0.0.0:6142".parse().unwrap();

    // TCP listener
    let listener = TcpListener::bind(&addr).unwrap();

    // Server is coming
    let server = listener.incoming().for_each(move |socket| {
        // Pass socket to the process
        process(socket, state.clone());
        Ok(())
    }).map_err(|err| {
        println!("accept error = {:?}", err);
    });
    
    println!("Transfer Station at .6142");
    tokio::run(server);
}

