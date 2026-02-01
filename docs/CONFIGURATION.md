# Kusanagi Configuration Guide

## Overview

Kusanagi uses a structured configuration system based on the `config` crate. Configuration can be provided via:

1. **Environment variables** (highest priority)
2. **`kusanagi.toml`** in the current directory
3. **`$HOME/.config/kusanagi/kusanagi.toml`**
4. **Default values** (lowest priority)

## Quick Start

### Using Environment Variables

All environment variables are prefixed with `KUSANAGI_`:

```bash
# Server configuration
export KUSANAGI_SERVER_PORT=8080
export KUSANAGI_SERVER_HOST=0.0.0.0

# Prometheus
export KUSANAGI_PROMETHEUS_URL=http://prometheus:9090

# Enable dev mode
export KUSANAGI_DEV_MODE=true
```

### Using Configuration File

Create `kusanagi.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
timeout_secs = 30

[prometheus]
url = "http://prometheus:9090"
url_ha = "http://prometheus-ha:9090"  # Optional
cache_ttl_secs = 60

[integrations.ollama]
url = "http://ollama:11434/api/generate"
model = "ministral-3:14b"

[cache]
default_ttl_secs = 300
news_ttl_mins = 30

[log]
level = "info"
format = "pretty"
```

## Configuration Reference

### Server (`server`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_SERVER_HOST` | `0.0.0.0` | Bind address |
| `KUSANAGI_SERVER_PORT` | `8080` | Port to listen on |
| `KUSANAGI_SERVER_WORKERS` | `auto` | Number of worker threads |
| `KUSANAGI_SERVER_TIMEOUT_SECS` | `30` | Request timeout |
| `KUSANAGI_SERVER_KEEP_ALIVE_SECS` | `5` | Keep-alive timeout |

### Kubernetes (`kubernetes`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_KUBERNETES_ENABLED` | `true` | Enable K8s integration |
| `KUSANAGI_KUBERNETES_KUBECONFIG` | `None` | Path to kubeconfig |
| `KUSANAGI_KUBERNETES_NAMESPACE` | `default` | Default namespace |
| `KUSANAGI_KUBERNETES_ARGOCD_URL` | `None` | ArgoCD URL |
| `KUSANAGI_KUBERNETES_TIMEOUT_SECS` | `30` | API timeout |

### Prometheus (`prometheus`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_PROMETHEUS_URL` | `http://kube-prometheus-stack-prometheus...` | Prometheus URL |
| `KUSANAGI_PROMETHEUS_URL_HA` | `None` | Home Assistant Prometheus URL |
| `KUSANAGI_PROMETHEUS_USERNAME` | `None` | Basic auth username |
| `KUSANAGI_PROMETHEUS_PASSWORD` | `None` | Basic auth password |
| `KUSANAGI_PROMETHEUS_TIMEOUT_SECS` | `10` | Query timeout |
| `KUSANAGI_PROMETHEUS_CACHE_TTL_SECS` | `60` | Cache TTL |

### Alertmanager (`alertmanager`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_ALERTMANAGER_URL` | `http://kube-prometheus-stack-alertmanager...` | Alertmanager URL |
| `KUSANAGI_ALERTMANAGER_CACHE_TTL_SECS` | `60` | Cache TTL |

### Integrations

#### MCP Servers (`integrations.mcp`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_MCP_KUBERNETES_URL` | `http://localhost:3000/mcp/kubernetes` | Kubernetes MCP |
| `KUSANAGI_INTEGRATIONS_MCP_CILIUM_URL` | `http://localhost:3000/mcp/cilium` | Cilium MCP |
| `KUSANAGI_INTEGRATIONS_MCP_STEAMPIPE_URL` | `http://localhost:3000/mcp/steampipe` | Steampipe MCP |
| `KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL` | `http://localhost:3000/mcp/trivy` | Trivy MCP |

#### OpenObserve (`integrations.openobserve`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_OPENOBSERVE_ENDPOINT` | `None` | OpenObserve URL |
| `KUSANAGI_INTEGRATIONS_OPENOBSERVE_AUTH` | `None` | Auth token |
| `KUSANAGI_INTEGRATIONS_OPENOBSERVE_SAMPLE_RATE` | `1.0` | Sample rate (0.0-1.0) |

#### Home Assistant (`integrations.home_assistant`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_HOME_ASSISTANT_URL` | `None` | HA URL |
| `KUSANAGI_INTEGRATIONS_HOME_ASSISTANT_TOKEN` | `None` | Access token |
| `KUSANAGI_INTEGRATIONS_HOME_ASSISTANT_USERNAME` | `None` | Username (legacy) |
| `KUSANAGI_INTEGRATIONS_HOME_ASSISTANT_PASSWORD` | `None` | Password (legacy) |

#### Proxmox (`integrations.proxmox`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_PROXMOX_URLS` | `None` | Comma-separated URLs |
| `KUSANAGI_INTEGRATIONS_PROXMOX_USER` | `None` | API user |
| `KUSANAGI_INTEGRATIONS_PROXMOX_PASSWORD` | `None` | Password |
| `KUSANAGI_INTEGRATIONS_PROXMOX_TOKEN_ID` | `None` | Token ID |
| `KUSANAGI_INTEGRATIONS_PROXMOX_TOKEN_SECRET` | `None` | Token secret |

#### MQTT (`integrations.mqtt`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_MQTT_HOST` | `None` | Broker host:port |
| `KUSANAGI_INTEGRATIONS_MQTT_CLIENT_ID` | `kusanagi` | Client ID |
| `KUSANAGI_INTEGRATIONS_MQTT_USERNAME` | `None` | Username |
| `KUSANAGI_INTEGRATIONS_MQTT_PASSWORD` | `None` | Password |

#### Weather (`integrations.weather`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_WEATHER_API_KEY` | `None` | OpenWeatherMap API key |
| `KUSANAGI_INTEGRATIONS_WEATHER_CITIES` | `Lyon,Mexico City,New York` | Comma-separated cities |
| `KUSANAGI_INTEGRATIONS_WEATHER_UPDATE_INTERVAL_MINS` | `30` | Update interval |

#### Calendar (`integrations.calendar`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_CALENDAR_GOOGLE_API_KEY` | `None` | Google API key |
| `KUSANAGI_INTEGRATIONS_CALENDAR_GOOGLE_CLIENT_SECRET` | `None` | Client secret |
| `KUSANAGI_INTEGRATIONS_CALENDAR_GOOGLE_REDIRECT_URL` | `None` | OAuth redirect URL |

#### Slack (`integrations.slack`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_SLACK_BOT_TOKEN` | `None` | Bot token |
| `KUSANAGI_INTEGRATIONS_SLACK_BOT_USER_ID` | `None` | Bot user ID |
| `KUSANAGI_INTEGRATIONS_SLACK_CHANNEL_ID` | `None` | Channel ID |
| `KUSANAGI_INTEGRATIONS_SLACK_SIGNING_SECRET` | `None` | Signing secret |

#### Ollama (`integrations.ollama`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_INTEGRATIONS_OLLAMA_URL` | `http://192.168.0.52:11434/api/generate` | API URL |
| `KUSANAGI_INTEGRATIONS_OLLAMA_MODEL` | `ministral-3:14b` | Model name |

### Storage (`storage`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_STORAGE_ENDPOINT` | `None` | S3/MinIO endpoint |
| `KUSANAGI_STORAGE_BUCKET` | `None` | Bucket name |
| `KUSANAGI_STORAGE_ACCESS_KEY` | `None` | Access key |
| `KUSANAGI_STORAGE_SECRET_KEY` | `None` | Secret key |
| `KUSANAGI_STORAGE_REGION` | `us-east-1` | Region |

### Cache (`cache`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_CACHE_DEFAULT_TTL_SECS` | `300` | Default cache TTL |
| `KUSANAGI_CACHE_NEWS_TTL_MINS` | `30` | News feed cache TTL |
| `KUSANAGI_CACHE_PROMETHEUS_TTL_SECS` | `60` | Prometheus cache TTL |
| `KUSANAGI_CACHE_CILIUM_TTL_SECS` | `60` | Cilium cache TTL |

### Security (`security`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_SECURITY_AUTH_ENABLED` | `false` | Enable authentication |
| `KUSANAGI_SECURITY_JWT_SECRET` | `None` | JWT secret key |
| `KUSANAGI_SECURITY_SESSION_TIMEOUT_HOURS` | `24` | Session timeout |
| `KUSANAGI_SECURITY_CORS_ORIGINS` | `*` | Allowed CORS origins |

### Logging (`log`)

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_LOG_LEVEL` | `info` | Log level |
| `KUSANAGI_LOG_FORMAT` | `pretty` | Format (json/pretty) |
| `KUSANAGI_LOG_OPENTELEMETRY` | `false` | Enable OpenTelemetry |

### Development Mode

| Variable | Default | Description |
|----------|---------|-------------|
| `KUSANAGI_DEV_MODE` | `false` | Enable dev mode |

## Validation

The configuration is validated at startup. Invalid configurations will cause the application to exit with an error message.

### Validated Fields

- **Server port**: Must be > 0
- **Prometheus URL**: Must start with `http://` or `https://`
- **Timeouts**: Must be > 0
- **Log level**: Must be one of `trace`, `debug`, `info`, `warn`, `error`

## Example Configurations

### Minimal Development Setup

```toml
[server]
port = 8080

[prometheus]
url = "http://localhost:9090"

[log]
level = "debug"
```

### Production Setup

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8
timeout_secs = 30

[prometheus]
url = "http://prometheus.monitoring.svc:9090"
url_ha = "http://prometheus-ha.monitoring.svc:9090"
username = "admin"
password = "secret"

[integrations]

[integrations.slack]
bot_token = "xoxb-..."
channel_id = "C1234567890"

[integrations.ollama]
url = "http://ollama.ai.svc:11434/api/generate"
model = "llama2"

[cache]
default_ttl_secs = 60
news_ttl_mins = 15
prometheus_ttl_secs = 30

[security]
auth_enabled = true
jwt_secret = "your-secret-key-here"
cors_origins = "https://kusanagi.example.com"

[log]
level = "info"
format = "json"
opentelemetry = true
```

## Migration from Old Configuration

### Before (Environment variables only)

```bash
export PROMETHEUS_URL="http://prometheus:9090"
export OLLAMA_URL="http://ollama:11434/api/generate"
export OLLAMA_MODEL="ministral-3:14b"
```

### After (With prefix)

```bash
export KUSANAGI_PROMETHEUS_URL="http://prometheus:9090"
export KUSANAGI_INTEGRATIONS_OLLAMA_URL="http://ollama:11434/api/generate"
export KUSANAGI_INTEGRATIONS_OLLAMA_MODEL="ministral-3:14b"
```

## Troubleshooting

### Configuration Not Loading

1. Check that environment variables use the `KUSANAGI_` prefix
2. Verify TOML file syntax with `tomlv kusanagi.toml`
3. Check file permissions for config files

### Validation Errors

```
Error: Configuration error: Server port cannot be 0
```

Review the error message and adjust the configuration accordingly.

### Debug Configuration

Add to your config:

```toml
[log]
level = "debug"
```

Or set environment variable:

```bash
export KUSANAGI_LOG_LEVEL=debug
```

## Testing

Run configuration tests:

```bash
cargo test config::
```

This will test:
- Default values
- Validation logic
- Configuration loading
- Edge cases
