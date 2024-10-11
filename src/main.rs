use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crdt_websocket::{
    crdt::{CrdtCounter, GCounter, PNCounter},
    start_websocket_server,
};

#[tokio::main]
async fn main() {
    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();

    start_websocket_server(crdt, gcounter, pncounter, listener, None).await;
}
