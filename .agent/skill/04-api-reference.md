# API Reference

## Endpoints

### Weather
| Method | Endpoint | Handler |
|--------|----------|---------|
| GET | `/api/weather/current` | `get_weather_handler` |
| POST | `/api/weather/refresh` | `refresh_weather_handler` |

### HomeAssistant
| Method | Endpoint | Handler |
|--------|----------|---------|
| GET | `/api/ha/devices` | `get_devices_handler` |
| GET | `/api/ha/sensors` | `get_sensors_handler` |
| GET | `/api/ha/automations` | `get_automations_handler` |

### Security
| Method | Endpoint | Handler |
|--------|----------|---------|
| GET | `/api/security/summary` | `get_security_handler` |
| GET | `/api/security/vulnerabilities` | `get_vulnerabilities_handler` |
| GET | `/api/security/reports` | `get_security_reports_handler` |

### Alerts
| Method | Endpoint | Handler |
|--------|----------|---------|
| GET | `/api/alerts` | `get_alerts_handler` |

### Kubernetes
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/services` | List services (cached 3min) |
| GET | `/api/ingress` | List ingress (cached 3min) |
| GET | `/api/nodes/status` | Node status |
| GET | `/api/pods/status` | Pod status |
| GET | `/api/storage` | Storage info |

## Response Format
All endpoints return JSON. Error responses include empty data arrays.

```json
// Success
{"data": [...], "count": 10}

// Error
{"error": "message", "data": [], "count": 0}
```

## Cache Headers
- Services/Ingress: 3 min TTL
- Weather: 1 hour TTL
- No-cache on force refresh
