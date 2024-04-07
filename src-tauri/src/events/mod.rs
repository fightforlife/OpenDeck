pub mod frontend;
pub mod inbound;
pub mod outbound;

use inbound::RegisterEvent;

use std::collections::HashMap;

use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

type Sockets = Lazy<Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>>>;
static PLUGIN_SOCKETS: Sockets = Lazy::new(|| Mutex::new(HashMap::new()));
static PROPERTY_INSPECTOR_SOCKETS: Sockets = Lazy::new(|| Mutex::new(HashMap::new()));
static PLUGIN_QUEUES: Lazy<Mutex<HashMap<String, Vec<Message>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PROPERTY_INSPECTOR_QUEUES: Lazy<Mutex<HashMap<String, Vec<Message>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a plugin or property inspector to send and receive events with its WebSocket.
pub async fn register_plugin(event: RegisterEvent, stream: WebSocketStream<TcpStream>) {
	let (mut read, write) = stream.split();
	match event {
		RegisterEvent::RegisterPlugin { uuid } => {
			if let Some(queue) = PLUGIN_QUEUES.lock().await.get(&uuid) {
				for message in queue {
					let _ = read.feed(message.clone()).await;
				}
				let _ = read.flush().await;
			}
			PLUGIN_SOCKETS.lock().await.insert(uuid, read);
			tokio::spawn(write.try_for_each(inbound::process_incoming_message));
		}
		RegisterEvent::RegisterPropertyInspector { uuid } => {
			if let Some(queue) = PROPERTY_INSPECTOR_QUEUES.lock().await.get(&uuid) {
				for message in queue {
					let _ = read.feed(message.clone()).await;
				}
				let _ = read.flush().await;
			}
			PROPERTY_INSPECTOR_SOCKETS.lock().await.insert(uuid, read);
			tokio::spawn(write.try_for_each(inbound::process_incoming_message_pi));
		}
	};
}
