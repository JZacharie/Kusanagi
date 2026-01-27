use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::time::Duration;
use tokio;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let host = env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("MQTT_USER").ok();
    let pass = env::var("MQTT_PASSWORD").ok();

    println!("Testing MQTT connection to: {}", host);
    if let Some(u) = &user {
        println!("Using user: {}", u);
    }

    let client_id = "kusanagi-debug-client";
    let mut mqttoptions = MqttOptions::new(client_id, host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let (Some(u), Some(p)) = (user, pass) {
        mqttoptions.set_credentials(u, p);
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    tokio::spawn(async move {
        match client.subscribe("#", QoS::AtMostOnce).await {
            Ok(_) => println!("Successfully subscribed to #"),
            Err(e) => println!("Failed to subscribe: {}", e),
        }
    });

    println!("Waiting for messages (ctrl+c to stop)...");
    
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Received notification: {:?}", notification);
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    println!("Topic: {}, Payload: {}", publish.topic, String::from_utf8_lossy(&publish.payload));
                }
            }
            Err(e) => {
                eprintln!("MQTT EventLoop error: {}", e);
                break;
            }
        }
    }
}
