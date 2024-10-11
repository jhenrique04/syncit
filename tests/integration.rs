use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};
use tokio_tungstenite::tungstenite::Message;

use crdt_websocket::{
    crdt::{CrdtCounter, GCounter, PNCounter},
    start_websocket_server,
};

async fn setup_server(
    crdt: Arc<RwLock<CrdtCounter>>,
    gcounter: Arc<RwLock<GCounter>>,
    pncounter: Arc<RwLock<PNCounter>>,
) -> (String, oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (ready_tx, ready_rx) = oneshot::channel();

    let server_crdt = crdt.clone();
    let server_gcounter = gcounter.clone();
    let server_pncounter = pncounter.clone();

    tokio::spawn(async move {
        start_websocket_server(
            server_crdt,
            server_gcounter,
            server_pncounter,
            listener,
            Some(ready_tx),
        )
        .await;
    });

    // Return the server address and the receiver to wait for readiness
    (format!("ws://{}", addr), ready_rx)
}

#[tokio::test]
async fn test_crdt_counter_increment() {
    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let (url, ready_rx) = setup_server(crdt.clone(), gcounter.clone(), pncounter.clone()).await;
    ready_rx.await.unwrap();

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Failed to connect");

    // Send an increment_crdt command
    let increment_command = json!({ "command": "increment_crdt" }).to_string();
    ws_stream
        .send(Message::text(increment_command))
        .await
        .unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: CrdtCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.count, 1);
        }
        _ => panic!("Did not receive valid response for increment_crdt"),
    }

    // Send a get_crdt command
    let get_command = json!({ "command": "get_crdt" }).to_string();
    ws_stream.send(Message::text(get_command)).await.unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: CrdtCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.count, 1);
        }
        _ => panic!("Did not receive valid response for get_crdt"),
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn test_gcounter_increment() {
    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let (url, ready_rx) = setup_server(crdt.clone(), gcounter.clone(), pncounter.clone()).await;
    ready_rx.await.unwrap();

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Failed to connect");

    // Send an increment_gcounter command
    let increment_command = json!({ "command": "increment_gcounter" }).to_string();
    ws_stream
        .send(Message::text(increment_command))
        .await
        .unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: GCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.count, 1);
        }
        _ => panic!("Did not receive valid response for increment_gcounter"),
    }

    // Send a get_gcounter command
    let get_command = json!({ "command": "get_gcounter" }).to_string();
    ws_stream.send(Message::text(get_command)).await.unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: GCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.count, 1);
        }
        _ => panic!("Did not receive valid response for get_gcounter"),
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn test_pncounter_increment_decrement() {
    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let (url, ready_rx) = setup_server(crdt.clone(), gcounter.clone(), pncounter.clone()).await;
    ready_rx.await.unwrap();

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Failed to connect");

    // Increment PNCounter
    let increment_command = json!({ "command": "increment_pncounter" }).to_string();
    ws_stream
        .send(Message::text(increment_command))
        .await
        .unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: PNCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.value(), 1);
        }
        _ => panic!("Did not receive valid response for increment_pncounter"),
    }

    // Decrement PNCounter
    let decrement_command = json!({ "command": "decrement_pncounter" }).to_string();
    ws_stream
        .send(Message::text(decrement_command))
        .await
        .unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: PNCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.value(), 0);
        }
        _ => panic!("Did not receive valid response for decrement_pncounter"),
    }

    // Get PNCounter value
    let get_command = json!({ "command": "get_pncounter" }).to_string();
    ws_stream.send(Message::text(get_command)).await.unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: PNCounter = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response.value(), 0);
        }
        _ => panic!("Did not receive valid response for get_pncounter"),
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn test_unknown_command() {
    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let (url, ready_rx) = setup_server(crdt.clone(), gcounter.clone(), pncounter.clone()).await;
    ready_rx.await.unwrap();

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Failed to connect");

    // Send an unknown command
    let unknown_command = json!({ "command": "unknown_command" }).to_string();
    ws_stream
        .send(Message::text(unknown_command))
        .await
        .unwrap();

    match ws_stream.next().await {
        Some(Ok(Message::Text(response_text))) => {
            let response: serde_json::Value = serde_json::from_str(&response_text).unwrap();
            assert_eq!(response["error"], "Unknown command");
        }
        _ => panic!("Did not receive valid response for unknown_command"),
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_clients() {
    use rand::seq::SliceRandom;
    use rand::{rngs::SmallRng, SeedableRng};

    let num_clients = 10;
    let crdt_increments_per_client = 10;
    let gcounter_increments_per_client = 10;
    let pncounter_increments_per_client = 10;
    let pncounter_decrements_per_client = 5;

    let crdt = Arc::new(RwLock::new(CrdtCounter::new()));
    let gcounter = Arc::new(RwLock::new(GCounter::new()));
    let pncounter = Arc::new(RwLock::new(PNCounter::new()));

    let (url, ready_rx) = setup_server(crdt.clone(), gcounter.clone(), pncounter.clone()).await;
    ready_rx.await.unwrap();

    let client_tasks: Vec<_> = (0..num_clients)
        .map(|_| {
            let url_clone = url.clone();
            tokio::spawn(async move {
                let (mut ws_stream, _) =
                    tokio_tungstenite::connect_async(&url_clone).await.unwrap();

                let mut commands = Vec::new();

                for _ in 0..crdt_increments_per_client {
                    commands.push(json!({ "command": "increment_crdt" }).to_string());
                }
                for _ in 0..gcounter_increments_per_client {
                    commands.push(json!({ "command": "increment_gcounter" }).to_string());
                }
                for _ in 0..pncounter_increments_per_client {
                    commands.push(json!({ "command": "increment_pncounter" }).to_string());
                }
                for _ in 0..pncounter_decrements_per_client {
                    commands.push(json!({ "command": "decrement_pncounter" }).to_string());
                }

                // Shuffle commands to simulate random operation order
                let mut rng = SmallRng::from_entropy();
                commands.shuffle(&mut rng);

                for command in commands {
                    ws_stream.send(Message::text(command)).await.unwrap();
                    ws_stream.next().await;
                }

                ws_stream.close(None).await.unwrap();
            })
        })
        .collect();

    for client_task in client_tasks {
        client_task.await.unwrap();
    }

    // Compute expected values
    let expected_crdt_value = num_clients * crdt_increments_per_client;
    let expected_gcounter_value = num_clients * gcounter_increments_per_client;
    let expected_pncounter_value =
        num_clients * (pncounter_increments_per_client - pncounter_decrements_per_client);

    // Get the actual values from the counters
    let crdt_value = crdt.read().await.count;
    let gcounter_value = gcounter.read().await.count;
    let pncounter_value = pncounter.read().await.value();

    assert_eq!(
        crdt_value, expected_crdt_value,
        "CrdtCounter value mismatch"
    );
    assert_eq!(
        gcounter_value, expected_gcounter_value,
        "GCounter value mismatch"
    );
    assert_eq!(
        pncounter_value, expected_pncounter_value,
        "PNCounter value mismatch"
    );
}
