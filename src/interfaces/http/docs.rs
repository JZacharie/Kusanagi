use crate::interfaces::http::handlers::business::chat;
use crate::interfaces::http::handlers::core::{health, system};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        system::system_status,
        system::system_logs,
        chat::post_chat_handler,
    ),
    components(
        schemas(
            crate::domain::services::system_service::SystemStatus,
            chat::ChatRequest,
            chat::ChatResponse,
        )
    ),
    tags(
        (name = "kusanagi", description = "Kusanagi API")
    )
)]
pub struct ApiDoc;
