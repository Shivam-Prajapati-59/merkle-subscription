-- TABLE 1: Storage of Users (The Leaf Data)
CREATE TABLE subscriber_storage (
    wallet_address      VARCHAR(44) PRIMARY KEY, -- Base58 Solana Address
    expiration_ts       BIGINT NOT NULL,         -- Unix Timestamp (i64)
    last_updated_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- TABLE 2: Merkle State (The Global Program State)
CREATE TABLE merkle_state (
    id                  SERIAL PRIMARY KEY,
    root_hash           VARCHAR(64) NOT NULL,    -- Hex-encoded SHA256 Root
    is_synced_on_chain  BOOLEAN DEFAULT FALSE,
    tx_signature        VARCHAR(88),             -- Solana Tx Sig for tracking
    created_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);