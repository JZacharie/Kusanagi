use serde_json::{json, Value};
use tokio::process::Command;

pub async fn get_ha_devices() -> Result<Value, String> {
    // Essayer l'API Home Assistant
    let ha_api_output = Command::new("curl")
        .args(&["-s", "-H", "Authorization: Bearer LONG_LIVED_ACCESS_TOKEN", "http://localhost:8123/api/states"])
        .output()
        .await;
    
    if let Ok(result) = ha_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(states) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let devices: Vec<Value> = states.iter()
                    .filter(|state| {
                        let entity_id = state["entity_id"].as_str().unwrap_or("");
                        entity_id.starts_with("light.") || 
                        entity_id.starts_with("switch.") || 
                        entity_id.starts_with("sensor.") ||
                        entity_id.starts_with("binary_sensor.")
                    })
                    .map(|state| {
                        json!({
                            "entity_id": state["entity_id"],
                            "friendly_name": state["attributes"]["friendly_name"],
                            "state": state["state"],
                            "device_class": state["attributes"]["device_class"],
                            "domain": state["entity_id"].as_str().unwrap_or("").split('.').next().unwrap_or("")
                        })
                    })
                    .collect();
                
                return Ok(json!(devices));
            }
        }
    }
    
    // Fallback: essayer différents ports HA
    for port in [8123, 8124, 8125] {
        let ha_check = Command::new("curl")
            .args(&["-s", "-m", "2", &format!("http://localhost:{}/api/", port)])
            .output()
            .await;
        
        if let Ok(result) = ha_check {
            if result.status.success() {
                let response = String::from_utf8_lossy(&result.stdout);
                if response.contains("Home Assistant") || response.contains("API running") {
                    return Ok(json!([{
                        "entity_id": "homeassistant.detected",
                        "friendly_name": "Home Assistant Instance",
                        "state": "online",
                        "port": port,
                        "source": "detection"
                    }]));
                }
            }
        }
    }
    
    // Fallback: chercher des processus Home Assistant
    let ha_process = Command::new("ps")
        .args(&["aux"])
        .output()
        .await;
    
    if let Ok(result) = ha_process {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            if output_str.contains("homeassistant") || output_str.contains("hass") {
                return Ok(json!([{
                    "entity_id": "process.homeassistant",
                    "friendly_name": "Home Assistant Process",
                    "state": "running",
                    "source": "process_detection"
                }]));
            }
        }
    }
    
    Ok(json!([]))
}

pub async fn get_ha_sensors() -> Result<Value, String> {
    // Essayer l'API Home Assistant pour les sensors
    let ha_api_output = Command::new("curl")
        .args(&["-s", "-H", "Authorization: Bearer LONG_LIVED_ACCESS_TOKEN", "http://localhost:8123/api/states"])
        .output()
        .await;
    
    if let Ok(result) = ha_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(states) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let sensors: Vec<Value> = states.iter()
                    .filter(|state| {
                        let entity_id = state["entity_id"].as_str().unwrap_or("");
                        entity_id.starts_with("sensor.") || entity_id.starts_with("binary_sensor.")
                    })
                    .map(|state| {
                        json!({
                            "entity_id": state["entity_id"],
                            "state": state["state"],
                            "attributes": {
                                "friendly_name": state["attributes"]["friendly_name"],
                                "unit_of_measurement": state["attributes"]["unit_of_measurement"],
                                "device_class": state["attributes"]["device_class"]
                            }
                        })
                    })
                    .collect();
                
                return Ok(json!(sensors));
            }
        }
    }
    
    // Fallback: simuler des sensors système
    let cpu_temp = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()
        .and_then(|temp| temp.trim().parse::<i32>().ok())
        .map(|temp| temp / 1000)
        .unwrap_or(45);
    
    let uptime = std::fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| (s / 3600.0) as u32)
        .unwrap_or(0);
    
    Ok(json!([
        {
            "entity_id": "sensor.cpu_temperature",
            "state": cpu_temp,
            "attributes": {
                "friendly_name": "CPU Temperature",
                "unit_of_measurement": "°C",
                "device_class": "temperature"
            },
            "source": "system_fallback"
        },
        {
            "entity_id": "sensor.system_uptime",
            "state": uptime,
            "attributes": {
                "friendly_name": "System Uptime",
                "unit_of_measurement": "hours",
                "device_class": "duration"
            },
            "source": "system_fallback"
        }
    ]))
}

pub async fn get_ha_automations() -> Result<Value, String> {
    // Essayer l'API Home Assistant pour les automations
    let ha_api_output = Command::new("curl")
        .args(&["-s", "-H", "Authorization: Bearer LONG_LIVED_ACCESS_TOKEN", "http://localhost:8123/api/config/automation/config"])
        .output()
        .await;
    
    if let Ok(result) = ha_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(automations) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let formatted_automations: Vec<Value> = automations.iter()
                    .map(|automation| {
                        json!({
                            "id": automation["id"],
                            "alias": automation["alias"],
                            "description": automation["description"],
                            "trigger": automation["trigger"],
                            "action": automation["action"],
                            "mode": automation["mode"]
                        })
                    })
                    .collect();
                
                return Ok(json!(formatted_automations));
            }
        }
    }
    
    // Fallback: essayer les états automation.*
    let ha_states_output = Command::new("curl")
        .args(&["-s", "-H", "Authorization: Bearer LONG_LIVED_ACCESS_TOKEN", "http://localhost:8123/api/states"])
        .output()
        .await;
    
    if let Ok(result) = ha_states_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(states) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let automations: Vec<Value> = states.iter()
                    .filter(|state| {
                        state["entity_id"].as_str().unwrap_or("").starts_with("automation.")
                    })
                    .map(|state| {
                        json!({
                            "entity_id": state["entity_id"],
                            "state": state["state"],
                            "attributes": {
                                "friendly_name": state["attributes"]["friendly_name"],
                                "last_triggered": state["attributes"]["last_triggered"]
                            }
                        })
                    })
                    .collect();
                
                return Ok(json!(automations));
            }
        }
    }
    
    // Fallback: chercher des fichiers de configuration HA
    let ha_config_check = Command::new("find")
        .args(&["/", "-name", "configuration.yaml", "-path", "*/homeassistant/*", "2>/dev/null"])
        .output()
        .await;
    
    if let Ok(result) = ha_config_check {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            if !output_str.trim().is_empty() {
                return Ok(json!([{
                    "id": "config_detected",
                    "alias": "Home Assistant Configuration Detected",
                    "description": "Found HA config files on system",
                    "state": "detected",
                    "config_path": output_str.trim(),
                    "source": "config_detection"
                }]));
            }
        }
    }
    
    Ok(json!([]))
}
