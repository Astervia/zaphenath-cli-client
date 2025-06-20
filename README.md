# Zaph CLI

> A command-line interface for interacting with the [Zaphenath](https://zaphenath.astervia.tech) smart contract — a protocol for secure, time-locked data access based on user inactivity.

## 📦 Installation

### Prerequisites

- [Rust](https://rust-lang.org/tools/install)
- An Ethereum-compatible RPC endpoint (e.g., Anvil, Infura)
- A deployed Zaphenath contract (or use `--mock` mode for local testing)

### Clone and Build

```bash
git clone https://github.com/Astervia/zaphenath-cli-client.git
cd zaph-cli
cargo build --release
```

### Optionally, install globally

```bash
cargo install --path .
```

Once installed, you can invoke the CLI via:

```bash
zaph --help
```

## 🧠 About Zaphenath

Zaphenath is a Solidity-based smart contract system for securely storing and revealing sensitive data (like wills, contingency plans, or secrets) after a period of user inactivity.

The core logic is based on:

- **Encrypted data keys** linked to an owner
- **Timeouts** that define when custodians may access data
- **Pings** to prove liveness and keep data private
- **Role-based access control** for delegated custody

📖 [Read the whitepaper](https://zaphenath.astervia.tech/whitepaper)

## 🧰 CLI Command Structure

```bash
zaph <COMMAND> [OPTIONS]
```

### Top-level commands

| Command    | Description                                 |
| ---------- | ------------------------------------------- |
| `config`   | Manage local configuration and key metadata |
| `contract` | Interact directly with the smart contract   |
| `daemon`   | Run a background service to auto-ping keys  |

## 🛠 Basic Usage

### 1. Initialize your config

```bash
zaph config init
```

This creates a local config file (e.g., `~/.config/zaphenath/config.json`) used for storing key metadata.

### 2. Create a key

```bash
zaph contract create-key \
  --key-id my-will \
  --data deadbeefcafebabe \
  --timeout 604800 \
  --contract-address 0xYourZaphenathAddress \
  --private-key-path ./my-key.hex \
  --rpc-url http://localhost:8545 \
  --yes
```

- `--data` is a hex-encoded string (your encrypted payload)
- `--timeout` is in seconds (e.g. 7 days = 604800)

### 3. Ping a key to keep it private

```bash
zaph contract ping-key --key-id my-will --yes
```

### 4. Read key data

```bash
zaph contract read-key --key-id my-will --decode
```

Use `--decode` to attempt UTF-8 decoding of the hex data.

### 5. Assign a custodian

```bash
zaph contract set-custodian \
  --key-id my-will \
  --user-address 0xFriend \
  --role Reader \
  --can-ping true
```

## 🌀 Daemon Usage

The daemon can automatically ping all keys in your config on a schedule:

### Run in foreground

```bash
zaph daemon run --interval 60
```

### Run in detached mode (background)

```bash
zaph daemon run --interval 60 --detached
```

### Stop the daemon

```bash
zaph daemon stop
```

### View logs

```bash
zaph daemon logs
```

## 🔍 Configuration File

The config file stores keys you've created or imported. Each entry contains:

```json
{
  "key_id": "my-will",
  "owner": "0xYourAddress",
  "contract_address": "0x...",
  "private_key_path": "./my-key.hex",
  "network": "anvil",
  "rpc_url": "http://localhost:8545",
  "timeout": 604800,
  "custodians": [],
  "last_ping_timestamp": 1721019123
}
```

You can override the config path with:

```bash
zaph --config /path/to/custom_config.json ...
```

## 🧪 Mock Mode for Testing

Use `--mock` to skip actual blockchain interaction and simulate behavior:

```bash
zaph contract create-key --mock ...
zaph daemon run --mock ...
```

## 🧱 Project Structure

```
.
├── src
│   ├── main.rs               # CLI entrypoint
│   ├── cmd/                  # CLI command implementations
│   ├── contract/             # Ethereum interaction logic
│   ├── config.rs             # Config loading/saving
│   └── ...
├── abi/Zaphenath.json        # ABI definition
├── tests/                    # Integration tests
├── Makefile                  # Dev/test helpers
└── Cargo.toml
```

## 🛡 Safety Tips

- Always use a private key stored securely (consider using encrypted keystores)
- Back up your config files and encrypted data
- Test in mock mode or on a testnet before using mainnet
