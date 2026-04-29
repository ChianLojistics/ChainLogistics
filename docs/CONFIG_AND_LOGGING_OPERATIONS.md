# Configuration and Logging Operations

## Environment Configuration Management

Backend configuration is now managed as code with profile-based files:

- `backend/config/default.toml`
- `backend/config/development.toml`
- `backend/config/production.toml`

Runtime profile is selected via `CHAINLOGISTICS_ENV` (fallback: `APP_ENV`).

Environment overrides support nested keys using:

- `CHAINLOGISTICS__<SECTION>__<KEY>=<value>`

Examples:

- `CHAINLOGISTICS__SERVER__PORT=3001`
- `CHAINLOGISTICS__SECURITY__ENFORCE_HTTPS=true`

Validation is enforced during startup for:

- non-empty `database.url` and `redis.url`
- `jwt_secret` minimum length
- `encryption_key` exact length (32)
- TLS cert/key required when `tls_enabled=true`

## Centralized Log Aggregation

`docker-compose.yml` includes:

- `loki` for centralized log storage
- `promtail` for log shipping
- `grafana` datasource for Loki log analysis

Structured logging is enforced in backend via JSON logs by default:

- `LOG_FORMAT=json` (default)
- `LOG_FORMAT=pretty` for local human-readable output

Retention policy is defined in Loki config:

- `docker/loki/loki-config.yaml`
- `limits_config.retention_period: 720h` (30 days)
