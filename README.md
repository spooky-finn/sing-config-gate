# Sing-Box Config Bot (Rust)

A Telegram bot for managing sing-box VPN configurations

## Features

- User registration via Telegram
- Admin approval workflow for new users
- Automatic VPN UUID generation for accepted users
- sing-box server config generation with all accepted users
- Remote deployment utility via SSH
- SQLite database with Diesel ORM

## Project Structure

```
rust/
├── src/
│   ├── adapters/       # External implementations (database, etc.)
│   ├── db/             # Database schema and types
│   │   ├── enums.rs    # User status enum
│   │   ├── mod.rs      # User and VpnUuid models
│   │   └── schema.rs   # Diesel schema definitions
│   ├── ports/          # Interface definitions (ports)
│   ├── service/        # Business logic
│   ├── utils/          # Utilities (logging, config)
│   ├── main.rs         # Bot entry point
│   ├── deploy.rs       # Deployment utility
│   └── generate_config.rs  # Config generator
├── db/
│   └── migrations/     # Database migrations
├── config/
│   ├── sing-box.template.json  # Template config
│   └── sing-box.server.json    # Generated server config (gitignored)
├── Cargo.toml
└── .env.example
```

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SQLite3
- libssh2 (for deployment utility)

## Installation

1. Clone the repository and navigate to the Rust directory:
   ```bash
   cd rust
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Edit `.env` and fill in your configuration values.

4. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

### Application Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `TG_BOT_TOKEN` | Telegram bot token | Yes |
| `TG_ADMIN_ID` | Telegram admin user ID | Yes |
| `CLIENT_CONFIG_ENDPOINT` | Base URL for config downloads | Yes |
| `DB_LOCATION` | SQLite database path | No (default: `./db/vpn_signaling_server.db`) |
| `LOG_LEVEL` | Logging level | No (default: `info`) |
| `LOG_DISABLE_TIMESTAMP` | Disable timestamps in logs | No (default: `false`) |

### Sing-Box Server Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `SING_BOX_PRIVATE_KEY` | Reality private key | Yes |
| `SING_BOX_SHORT_ID` | Reality short ID | Yes |
| `SING_BOX_SERVER_NAME` | Server name for Reality (e.g., google.com) | No (default: google.com) |
| `SING_BOX_SERVER_PORT` | Server port | No (default: 443) |

### Deployment Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DEPLOY_HOST` | Remote server hostname | Yes |
| `DEPLOY_KEYFILE` | SSH private key path | Yes |
| `DEPLOY_USER` | SSH username | Yes |
| `DEPLOY_COMMAND` | Command to execute on remote (e.g., `systemctl restart sing-box`) | Yes |
| `DEPLOY_CWD` | Working directory on remote | Yes |

## Usage

### Running the Bot

```bash
cargo run --bin bot
```

Or in release mode:

```bash
cargo run --release --bin bot
```

### Generating Server Config

This generates a sing-box server config with all accepted users:

```bash
cargo run --bin generate-config
```

The config is written to `config/sing-box.server.json`.

### Deploying to Remote Server

This uploads the generated config to the remote server and restarts sing-box:

```bash
cargo run --bin deploy
```

### Complete Workflow

1. **Start the bot**: Users can register via Telegram
2. **Admin approves users**: Admin clicks "Accept" in Telegram
3. **Generate config**: Run `cargo run --bin generate-config` to create server config with all accepted users
4. **Deploy**: Run `cargo run --bin deploy` to upload config and restart sing-box on the server

## Development

### Format code

```bash
cargo fmt
```

### Check for errors

```bash
cargo check
```

### Run tests

```bash
cargo test
```

### Build for production

```bash
cargo build --release
```

## Database Schema

### user table

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Telegram user ID (primary key) |
| username | TEXT | Telegram username |
| status | INTEGER | 0=New, 1=Accepted, 2=Rejected |
| auth_key | TEXT | Authentication key |
| created_at | TEXT | ISO 8601 timestamp |

### vless_identity table

| Column | Type | Description |
|--------|------|-------------|
| uuid | TEXT | VLESS identity UUID (primary key) |
| user_id | INTEGER | Foreign key to user.id |

## Architecture

This project follows the **Ports and Adapters** (Hexagonal) architecture:

- **Ports**: Define interfaces for external interactions (database)
- **Adapters**: Implement the ports with concrete technologies (Diesel + SQLite)
- **Services**: Contain business logic, depend only on ports

## License

ISC
