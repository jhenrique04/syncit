use crate::crdt::{CrdtCounter, GCounter, PNCounter};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn start_websocket_server(
    crdt: Arc<RwLock<CrdtCounter>>,
    gcounter: Arc<RwLock<GCounter>>,
    pncounter: Arc<RwLock<PNCounter>>,
    listener: TcpListener,
    ready_tx: Option<oneshot::Sender<()>>,
) {
    println!(
        "Starting WebSocket server on {}",
        listener.local_addr().unwrap()
    );

    // Signal that the server is ready
    if let Some(ready_tx) = ready_tx {
        let _ = ready_tx.send(());
    }

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("Client connected");
                let crdt = crdt.clone();
                let gcounter = gcounter.clone();
                let pncounter = pncounter.clone();
                tokio::spawn(handle_connection(stream, crdt, gcounter, pncounter));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    crdt: Arc<RwLock<CrdtCounter>>,
    gcounter: Arc<RwLock<GCounter>>,
    pncounter: Arc<RwLock<PNCounter>>,
) {
    println!("Handling new connection");
    let ws_stream = match accept_async(stream).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Error during websocket handshake: {:?}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_text() {
            let text = msg.to_text().unwrap();
            println!("Received message: {}", text);

            if !text.is_empty() {
                if let Err(e) =
                    process_message(text, &crdt, &gcounter, &pncounter, &mut ws_sender).await
                {
                    eprintln!("Failed to process message: {:?}", e);
                }
            } else {
                eprintln!("Received empty message, ignoring...");
            }
        }
    }
}

async fn process_message(
    text: &str,
    crdt: &Arc<RwLock<CrdtCounter>>,
    gcounter: &Arc<RwLock<GCounter>>,
    pncounter: &Arc<RwLock<PNCounter>>,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing message: {}", text);
    let json_message: Value = serde_json::from_str(text)?;

    match json_message.get("command").and_then(|v| v.as_str()) {
        Some("increment_crdt") => handle_crdt(crdt, ws_sender, true).await,
        Some("reset_crdt") => handle_crdt(crdt, ws_sender, false).await,
        Some("get_crdt") => send_response(&*crdt.read().await, ws_sender).await,

        Some("increment_gcounter") => handle_gcounter(gcounter, ws_sender).await,
        Some("get_gcounter") => send_response(&*gcounter.read().await, ws_sender).await,

        Some("increment_pncounter") => handle_pncounter(pncounter, ws_sender, true).await,
        Some("decrement_pncounter") => handle_pncounter(pncounter, ws_sender, false).await,
        Some("get_pncounter") => send_response(&*pncounter.read().await, ws_sender).await,

        _ => {
            eprintln!("Unknown command: {}", json_message);
            send_error_response("Unknown command".to_string(), ws_sender).await
        }
    }
}

async fn handle_crdt(
    crdt: &Arc<RwLock<CrdtCounter>>,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
    increment: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut counter = crdt.write().await;
    if increment {
        counter.increment();
        println!("Incremented CrdtCounter: {}", counter.count);
    } else {
        counter.count = 0;
        println!("Reset CrdtCounter");
    }
    send_response(&*counter, ws_sender).await
}

async fn handle_gcounter(
    gcounter: &Arc<RwLock<GCounter>>,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut counter = gcounter.write().await;
    counter.increment();
    println!("Incremented GCounter: {}", counter.count);
    send_response(&*counter, ws_sender).await
}

async fn handle_pncounter(
    pncounter: &Arc<RwLock<PNCounter>>,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
    increment: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut counter = pncounter.write().await;
    if increment {
        counter.increment();
        println!("Incremented PNCounter: {}", counter.value());
    } else {
        counter.decrement();
        println!("Decremented PNCounter: {}", counter.value());
    }
    send_response(&*counter, ws_sender).await
}

async fn send_response<T: serde::Serialize>(
    response: &T,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let response_text = serde_json::to_string(response)?;
    println!("Sending response: {}", response_text);
    ws_sender.send(Message::text(response_text)).await?;
    Ok(())
}

async fn send_error_response(
    error_message: String,
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let error_response = serde_json::json!({
        "error": error_message,
    });
    println!("Sending error response: {}", error_response);
    ws_sender
        .send(Message::text(error_response.to_string()))
        .await?;
    Ok(())
}
