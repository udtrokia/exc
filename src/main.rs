
extern crate ex;

use ex::tokio;
use ex::tokio::net::{TcpListener, TcpStream};
use ex::tokio::prelude::{Stream, Future};
use ex::futures::future;
use ex::futures::future::Either;
use ex::shared::Shared;
use ex::lines::Lines;
use ex::peer::Peer;
use std::sync::{Arc, Mutex};

fn process(socket: TcpStream, state: Arc<Mutex<Shared>>) {
    let lines = Lines::new(socket);
    let connection = lines.into_future()
        .map_err(|(e, _)| e)
        .and_then(|(name, lines)| {
            let name = match name {
                Some(name) => name,
                None => {
                    return Either::A(future::ok(()))
                }
            };

            println!("`{:?}` is joining the chat", name);

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
    let addr = "127.0.0.1:6142".parse().unwrap();

    // TCP listener
    let listener = TcpListener::bind(&addr).unwrap();

    // Server is coming
    let server = listener.incoming().for_each(move |socket| {
        process(socket, state.clone());
        Ok(())
    }).map_err(|err| {
        println!("accept error = {:?}", err);
    });
    
    println!("server running on localhost:6142");
    tokio::run(server);
}
