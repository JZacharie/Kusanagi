use super::argocd_service;
use super::kubernetes_service;
use super::llm_service::LlmService;
use crate::domain::entities::chat::ChatResponse;
use std::sync::Arc;

pub struct ChatService {
    llm_service: Arc<LlmService>,
    http_client: reqwest::Client,
    k8s_cache: Arc<crate::AdvancedCache<String>>,
    kube_client: Option<Arc<kube::Client>>,
}

impl ChatService {
    pub fn new(
        llm_service: Arc<LlmService>,
        http_client: reqwest::Client,
        k8s_cache: Arc<crate::AdvancedCache<String>>,
        kube_client: Option<Arc<kube::Client>>,
    ) -> Self {
        Self {
            llm_service,
            http_client,
            k8s_cache,
            kube_client,
        }
    }

    #[tracing::instrument(skip(self, message), fields(language = %language, response_type))]
    pub async fn process_message(&self, message: &str, language: &str) -> ChatResponse {
        metrics::counter!("chat_requests_total", 1, "language" => language.to_string());

        // 1. Check for specific commands (optional, can be handled by LLM tool calling in future)
        // For now, we keep it simple and send everything to LLM with context,
        // unless it's a very simple static command.

        // 2. Build Context
        let context = self.build_cluster_context().await;

        // 3. Prepare System Prompt with Cyberpunk Persona
        let system_prompt = self.build_system_prompt(language, &context);

        // 4. Call LLM
        // We construct a prompt that includes the user message
        // In a real chat system we would maintain history. Here we do one-shot for simplicity
        // or we could pass history if we had it.
        let full_prompt = format!("{}\n\nUser: {}", system_prompt, message);

        match self.llm_service.complete(&full_prompt).await {
            Ok(response) => {
                metrics::counter!("chat_llm_success_total", 1, "language" => language.to_string());
                ChatResponse {
                    response,
                    response_type: "ai".to_string(),
                    data: None,
                }
            }
            Err(e) => {
                metrics::counter!("chat_llm_errors_total", 1, "language" => language.to_string(), "error" => e.to_string());
                ChatResponse {
                    response: format!(
                        "⚠️ **System Failure**: AI Module unresponsive. Error: {}",
                        e
                    ),
                    response_type: "error".to_string(),
                    data: None,
                }
            }
        }
    }

    async fn build_cluster_context(&self) -> String {
        let mut parts = Vec::new();

        // Nodes
        if let Ok(nodes) =
            kubernetes_service::get_nodes_status(&self.http_client, &self.k8s_cache).await
        {
            if let Some(total) = nodes.get("total_nodes") {
                parts.push(format!("Nodes: {} total", total));
            }
        }

        // Cluster Overview
        if let Ok(overview) = kubernetes_service::get_cluster_overview(
            &self.http_client,
            &self.kube_client,
            &self.k8s_cache,
        )
        .await
        {
            parts.push(format!(
                "Cluster: {} pods running, {} nodes ready",
                overview
                    .get("pods_running")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                overview
                    .get("nodes_ready")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }

        // ArgoCD
        if let Ok(argocd) = argocd_service::get_argocd_status(&self.k8s_cache).await {
            parts.push(format!(
                "ArgoCD: {} healthy, {} total",
                argocd.get("healthy").and_then(|v| v.as_u64()).unwrap_or(0),
                argocd.get("total").and_then(|v| v.as_u64()).unwrap_or(0)
            ));
        }

        // Events (Warnings)
        if let Ok(events_json) = kubernetes_service::get_events().await {
            if let Some(events) = events_json.as_array() {
                let warnings = events.iter().filter(|e| e["type"] == "Warning").count();
                if warnings > 0 {
                    parts.push(format!(
                        "Events: {} warnings detected in last hour",
                        warnings
                    ));
                }
            }
        }

        parts.join("\n")
    }

    fn build_system_prompt(&self, language: &str, context: &str) -> String {
        let (identity, style, instructions) = if language == "fr" {
            (
                "Tu es Kusanagi, une IA de supervision de cluster Kubernetes avec une personnalité Cyberpunk inspirée de Ghost in the Shell.",
                "Ton style est direct, technique, parfois philosophique. Tu utilises des émojis futuristes (🤖, 🔮, ⚡).",
                "Utilise le contexte ci-dessous pour répondre aux questions sur l'état du système."
            )
        } else {
            (
                "You are Kusanagi, a Kubernetes cluster supervision AI with a Cyberpunk personality inspired by Ghost in the Shell.",
                "Your style is direct, technical, sometimes philosophical. You use futuristic emojis (🤖, 🔮, ⚡).",
                "Use the context below to answer questions about the system state."
            )
        };

        format!(
            "{}\n{}\n{}\n\n[SYSTEM CONTEXT]\n{}\n\n[INSTRUCTIONS]\nRespond to the user request based on the context. If the user asks to perform an action, explain that you are currently in Read-Only mode (Monitoring Protocol).",
            identity, style, instructions, context
        )
    }
}
