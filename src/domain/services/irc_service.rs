use futures::StreamExt;
use irc::client::prelude::*;
use std::env;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct IrcService {
    server: String,
    port: u16,
    channel: String,
    nickname: String,
    tx: Option<mpsc::UnboundedSender<String>>,
}

impl Default for IrcService {
    fn default() -> Self {
        Self::new()
    }
}

impl IrcService {
    pub fn new() -> Self {
        let server = env::var("IRC_SERVER")
            .unwrap_or_else(|_| "simple-irc-server.productivity.svc".to_string());
        let port = env::var("IRC_PORT")
            .unwrap_or_else(|_| "6667".to_string())
            .parse()
            .unwrap_or(6667);
        let channel = env::var("IRC_CHANNEL").unwrap_or_else(|_| "#posekafe".to_string());
        let nickname = env::var("IRC_NICKNAME").unwrap_or_else(|_| "kusanagi".to_string());

        info!(
            "💬 IRC: Configured for {}:{} channel {} as {}",
            server, port, channel, nickname
        );

        Self {
            server,
            port,
            channel,
            nickname,
            tx: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        let config = Config {
            nickname: Some(self.nickname.clone()),
            server: Some(self.server.clone()),
            port: Some(self.port),
            channels: vec![self.channel.clone()],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        self.tx = Some(tx);

        let channel = self.channel.clone();
        let _nickname = self.nickname.clone();

        tokio::spawn(async move {
            match Client::from_config(config).await {
                Ok(mut client) => {
                    if let Err(e) = client.identify() {
                        error!("IRC: Failed to identify: {}", e);
                        return;
                    }

                    info!("💬 IRC: Connected and joined {}", channel);

                    let mut stream = client.stream().unwrap();
                    let sender = client.sender();

                    loop {
                        tokio::select! {
                            Some(message) = stream.next() => {
                                if let Ok(msg) = message {
                                    if let Command::PING(server1, _) = msg.command {
                                        if let Err(e) = sender.send_pong(&server1) {
                                            error!("IRC: Failed to send PONG: {}", e);
                                        }
                                    }
                                }
                            }
                            Some(text) = rx.recv() => {
                                if let Err(e) = sender.send_privmsg(&channel, &text) {
                                    error!("IRC: Failed to send message: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("IRC: Failed to connect: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn send_alert(&self, title: &str, message: &str, severity: &str) -> bool {
        if self.tx.is_none() {
            warn!("IRC: Not connected, skipping alert");
            return false;
        }

        let emoji = match severity.to_lowercase().as_str() {
            "critical" | "error" => "🔴",
            "warning" => "🟠",
            "success" | "good" => "🟢",
            _ => "🔵",
        };

        let text = format!(
            "{} {} Alert: {} - {}",
            emoji,
            severity.to_uppercase(),
            title,
            message
        );

        self.post_message(&text).await
    }

    pub async fn post_message(&self, text: &str) -> bool {
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(text.to_string()) {
                error!("IRC: Failed to queue message: {}", e);
                return false;
            }
            true
        } else {
            warn!("IRC: Not connected, message not sent");
            false
        }
    }
}
