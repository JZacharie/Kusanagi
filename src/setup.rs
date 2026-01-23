use serde::{Serialize, Deserialize};
use actix_web::{get, post, web, HttpResponse, Responder};
use std::env;
use regex::Regex;

#[derive(Serialize)]
pub struct FeatureStatus {
    pub name: String,
    pub active: bool,
    pub missing_vars: Vec<String>,
}

#[derive(Serialize)]
pub struct SetupStatus {
    pub features: Vec<FeatureStatus>,
    pub env_variables: Vec<EnvVarDefinition>,
}

#[derive(Serialize)]
pub struct EnvVarDefinition {
    pub key: String,
    pub description: String,
    pub example: String,
    pub regex: String,
    pub is_secret: bool,
}

#[derive(Deserialize)]
pub struct ValidateRequest {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub message: String,
}

pub fn get_env_definitions() -> Vec<EnvVarDefinition> {
    vec![
        EnvVarDefinition {
            key: "PROXMOX_URL".to_string(),
            description: "URL of your Proxmox API (e.g., https://192.168.1.1:8006)".to_string(),
            example: "https://proxmox.local:8006".to_string(),
            regex: r"^https?://[a-zA-Z0-9\.-]+(:\d+)?/?$".to_string(),
            is_secret: false,
        },
        EnvVarDefinition {
            key: "PROXMOX_USER".to_string(),
            description: "Proxmox API User (root@pam or similar)".to_string(),
            example: "root@pam".to_string(),
            regex: r"^[a-zA-Z0-9_@\.-]+$".to_string(),
            is_secret: false,
        },
        EnvVarDefinition {
            key: "HOME_ASSISTANT_URL".to_string(),
            description: "URL of your Home Assistant instance".to_string(),
            example: "http://homeassistant.local:8123".to_string(),
            regex: r"^https?://[a-zA-Z0-9\.-]+(:\d+)?/?$".to_string(),
            is_secret: false,
        },
        EnvVarDefinition {
            key: "POSTGRES_URL".to_string(),
            description: "PostgreSQL Connection String".to_string(),
            example: "postgres://user:password@localhost:5432/dbname".to_string(),
            regex: r"^postgres://.*$".to_string(),
            is_secret: true,
        },
        EnvVarDefinition {
            key: "MQTT_HOST".to_string(),
            description: "MQTT Broker Hostname".to_string(),
            example: "mqtt.local".to_string(),
            regex: r"^[a-zA-Z0-9\.-]+$".to_string(),
            is_secret: false,
        },
    ]
}

#[get("/api/setup/status")]
pub async fn get_setup_status() -> impl Responder {
    let definitions = get_env_definitions();
    let mut features = Vec::new();

    // Proxmox Feature
    let proxmox_vars = vec!["PROXMOX_URL", "PROXMOX_USER", "PROXMOX_TOKEN_ID", "PROXMOX_TOKEN_SECRET"];
    features.push(check_feature("Proxmox Monitoring", proxmox_vars));

    // Home Assistant Feature
    let ha_vars = vec!["HOME_ASSISTANT_URL", "HOME_ASSISTANT_TOKEN"];
    features.push(check_feature("Home Assistant Integration", ha_vars));

    // PostgreSQL Feature
    let pg_vars = vec!["POSTGRES_URL"];
    features.push(check_feature("PostgreSQL Health", pg_vars));

    // MQTT Feature
    let mqtt_vars = vec!["MQTT_HOST"];
    features.push(check_feature("MQTT Messaging", mqtt_vars));

    HttpResponse::Ok().json(SetupStatus {
        features,
        env_variables: definitions,
    })
}

fn check_feature(name: &str, vars: Vec<&str>) -> FeatureStatus {
    let mut missing = Vec::new();
    for var in vars {
        if env::var(var).is_err() {
            missing.push(var.to_string());
        }
    }
    FeatureStatus {
        name: name.to_string(),
        active: missing.is_empty(),
        missing_vars: missing,
    }
}

#[post("/api/setup/validate")]
pub async fn validate_var(body: web::Json<ValidateRequest>) -> impl Responder {
    let definitions = get_env_definitions();
    let def = definitions.iter().find(|d| d.key == body.key);

    match def {
        Some(d) => {
            let re = Regex::new(&d.regex).unwrap();
            if re.is_match(&body.value) {
                HttpResponse::Ok().json(ValidateResponse {
                    valid: true,
                    message: "Format is valid".to_string(),
                })
            } else {
                HttpResponse::Ok().json(ValidateResponse {
                    valid: false,
                    message: format!("Invalid format for {}. Expected example: {}", d.key, d.example),
                })
            }
        }
        None => HttpResponse::BadRequest().json(ValidateResponse {
            valid: false,
            message: "Unknown environment variable".to_string(),
        }),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_setup_status);
    cfg.service(validate_var);
}
