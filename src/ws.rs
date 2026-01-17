use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::info;

use crate::{argocd, events, pods};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
/// How often to check for new alerts
const ALERT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// WebSocket notification message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotificationMessage {
    #[serde(rename = "alert")]
    Alert {
        severity: String,
        title: String,
        message: String,
        source: String,
        timestamp: String,
    },
    #[serde(rename = "stats_update")]
    StatsUpdate {
        argocd_issues: usize,
        error_pods: usize,
        warning_events: usize,
    },
    #[serde(rename = "connected")]
    Connected { message: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: String },
}

/// Internal message for sending notifications
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendNotification(pub NotificationMessage);

/// WebSocket connection actor
pub struct NotificationSession {
    /// Client must send ping at least once per CLIENT_TIMEOUT
    hb: Instant,
    /// Last known state for change detection
    last_argocd_issues: usize,
    last_error_pods: usize,
    last_warning_events: usize,
}

impl NotificationSession {
    pub fn new() -> Self {
        Self {
            hb: Instant::now(),
            last_argocd_issues: 0,
            last_error_pods: 0,
            last_warning_events: 0,
        }
    }

    /// Heartbeat to keep connection alive
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                info!("WebSocket client heartbeat failed, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    /// Check for alerts periodically
    fn check_alerts(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(ALERT_CHECK_INTERVAL, |act, ctx| {
            let addr = ctx.address();
            actix::spawn(async move {
                if let Some(notification) = check_for_new_alerts().await {
                    addr.do_send(SendNotification(notification));
                }
            });
        });
    }
}

impl Actor for NotificationSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WebSocket client connected");
        
        // Start heartbeat
        self.hb(ctx);
        
        // Start alert checking
        self.check_alerts(ctx);
        
        // Send welcome message
        let welcome = NotificationMessage::Connected {
            message: "Connected to Kusanagi notifications".to_string(),
        };
        if let Ok(json) = serde_json::to_string(&welcome) {
            ctx.text(json);
        }

        // Send initial stats
        let addr = ctx.address();
        actix::spawn(async move {
            if let Some(stats) = get_current_stats().await {
                addr.do_send(SendNotification(stats));
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket client disconnected");
    }
}

/// Handle messages from client
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Handle client commands if needed
                if text.trim() == "ping" {
                    let hb = NotificationMessage::Heartbeat {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    if let Ok(json) = serde_json::to_string(&hb) {
                        ctx.text(json);
                    }
                } else if text.trim() == "stats" {
                    // Request immediate stats update
                    let addr = ctx.address();
                    actix::spawn(async move {
                        if let Some(stats) = get_current_stats().await {
                            addr.do_send(SendNotification(stats));
                        }
                    });
                }
            }
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

/// Handle notification messages
impl Handler<SendNotification> for NotificationSession {
    type Result = ();

    fn handle(&mut self, msg: SendNotification, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}

/// WebSocket handshake endpoint
pub async fn ws_notifications(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(NotificationSession::new(), &req, stream)
}

/// Check for new alerts that should be sent to clients
async fn check_for_new_alerts() -> Option<NotificationMessage> {
    // Get current stats and check for critical issues
    let mut alerts = Vec::new();

    // Check ArgoCD status
    if let Ok(argocd_status) = argocd::get_argocd_status().await {
        if argocd_status.unhealthy > 0 {
            alerts.push(NotificationMessage::Alert {
                severity: "warning".to_string(),
                title: "ArgoCD Apps Unhealthy".to_string(),
                message: format!("{} applications need attention", argocd_status.unhealthy),
                source: "argocd".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    // Check pods in error
    if let Ok(pods_status) = pods::get_pods_status().await {
        if pods_status.error_pods > 0 {
            alerts.push(NotificationMessage::Alert {
                severity: "error".to_string(),
                title: "Pods in Error".to_string(),
                message: format!("{} pods are in error state", pods_status.error_pods),
                source: "pods".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    // Return first alert if any (we can batch later)
    alerts.into_iter().next()
}

/// Get current cluster stats for WebSocket update
async fn get_current_stats() -> Option<NotificationMessage> {
    let argocd_issues = argocd::get_argocd_status()
        .await
        .map(|s| s.unhealthy)
        .unwrap_or(0);

    let error_pods = pods::get_pods_status()
        .await
        .map(|s| s.error_pods)
        .unwrap_or(0);

    let warning_events = events::get_events(None)
        .await
        .map(|s| s.warning_count)
        .unwrap_or(0);

    Some(NotificationMessage::StatsUpdate {
        argocd_issues,
        error_pods,
        warning_events,
    })
}
