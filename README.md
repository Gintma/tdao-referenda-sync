# tdao-referenda-sync

> A Rust async service that fetches SubSquare referenda and publishes them as proposals on OpenSquare.

## Features

- **Automated Sync**: Fetch latest referenda from SubSquare every 30 minutes  
- **De-duplication**: Tracks which referenda have already been published to avoid duplicates  
- **On-chain Snapshot**: Captures block height snapshots for Polkadot before proposal creation  
- **Signature**: Signs each proposal payload with an sr25519 key derived from your mnemonic  
- **Persistence**: Stores processed referendum indices in PostgreSQL  
- **Configurable**: Environment-driven configuration via `.env`  
- **Logging**: Structured logs with configurable levels (INFO/DEBUG/WARN/ERROR)

## Prerequisites

- **Rust** (1.60+)  
- **PostgreSQL** (accessible via `POSTGRES_URL`)  
- **OpenSquare API** access (space must be configured)  
- **Subscan API Key** (for Polkadot metadata)  
- **Optional**: Docker & Docker Compose

---

## Environment Configuration

Copy `.env.example` to `.env` and fill in your settings:

```dotenv
# OpenSquare space identifier
OPEN_SQUARE_SPACE=twodao

# PostgreSQL connection string
POSTGRES_URL=postgres://tdao:password@127.0.0.1:5432/tdao

# Mnemonic for proposal signing
MNEMONIC="task cricket awkward dolphin and garage add photo weather always giraffe apple"

# Subscan API key for Polkadot metadata
SUBSCAN_API_KEY=35a441cb8b6447e5a68fb64e8b57d1cd

# Number of referenda to fetch per round
PAGE_SIZE=50

# Log level: trace, debug, info, warn, error
RUST_LOG=info
