# 🌲 Merkle Subscription System

A subscription management system using Merkle trees for efficient on-chain verification on Solana. This system allows you to manage thousands of subscribers while storing only a 32-byte root hash on-chain, drastically reducing storage costs and improving scalability.

**Key Benefits:**

- ✅ **Scalable**: Handle millions of subscribers with O(log n) verification
- ✅ **Cost-Efficient**: Only 32 bytes on-chain vs. 40 bytes per subscriber
- ✅ **Secure**: Cryptographically proven membership using SHA256
- ✅ **Privacy-Preserving**: Subscribers only reveal their data when needed

## ✨ Features

### Backend (Rust)

- 🗄️ **PostgreSQL Integration**: Store subscriber data with expiration timestamps
- 🌲 **Merkle Tree Generation**: Build cryptographic proofs for subscribers
- 🔗 **Solana Integration**: Automatically sync root hash to on-chain program
- ✅ **Off-chain Verification**: Test proofs before submitting on-chain
- 🔐 **Tampering Detection**: Verify data integrity

### Smart Contract (Anchor/Solana)

- 🚀 **Efficient Verification**: O(log n) proof verification
- ⏰ **Expiration Checking**: Automatically reject expired subscriptions
- 🔒 **Authority Control**: Only authorized wallet can update root
- 📦 **Minimal Storage**: 65 bytes total on-chain vs. 40n bytes

### Security

- 🛡️ **Cryptographic Proofs**: SHA256-based merkle proofs
- 🔍 **Data Integrity**: Detects any tampering with subscriber data
- 🎫 **Expiration Enforcement**: On-chain timestamp validation

## 🌳 How Merkle Trees Work

### Concept

A Merkle tree allows you to prove membership in a set without storing the entire set:

```
                Root Hash (stored on-chain)
                    /        \
                H(AB)        H(CD)
               /    \        /    \
            H(A)   H(B)   H(C)   H(D)
             |      |      |      |
          User1  User2  User3  User4
```

### Verification Process

1. **Build Tree**: Hash all subscribers → `H(pubkey + expiration)`
2. **Store Root**: Upload 32-byte root hash to Solana
3. **Generate Proof**: User requests proof for their pubkey
4. **Verify On-chain**: Submit proof to smart contract
   - Reconstruct path from leaf to root
   - Compare with stored root hash
   - ✅ Match = Valid subscriber

### Example

For 10,000 subscribers:

- **Traditional**: 400KB on-chain (40 bytes × 10k)
- **Merkle Tree**: 32 bytes on-chain + ~440 bytes proof
- **Savings**: 99.9% reduction in on-chain storage! 🎉

## 📦 Prerequisites

### System Requirements

- **Rust**: 1.75+ with Cargo
- **Node.js**: 18+ with npm/yarn
- **Solana CLI**: 1.18+
- **Anchor**: 0.30+
- **PostgreSQL**: 14+

### Rust Dependencies

```toml
solana-sdk = "3.0"
solana-client = "3.1"
anchor-lang = "0.30"
rs_merkle = "1.5"
sqlx = "0.8"
```

## 🚀 Installation

### 1. Clone Repository

```bash
git clone https://github.com/yourusername/merkle-subscription.git
cd merkle-subscription
```

### 2. Install Solana Tools

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
```

### 3. Setup Backend

```bash
cd backend

# Install dependencies
cargo build

# Setup PostgreSQL database
createdb merkle_subscription

# Run migrations
sqlx migrate run
```

### 4. Setup Solana Program

```bash
cd ../merkle-program

# Install dependencies
npm install

# Build program
anchor build
```

## ⚙️ Configuration

### 1. Backend Configuration

Create `backend/.env`:

```env
DATABASE_URL=postgresql://user:password@localhost/merkle_subscription
SOLANA_RPC_URL=http://localhost:8899
SOLANA_KEYPAIR_PATH=./backend-authority.json
```

### 2. Generate Authority Keypair

```bash
cd backend
solana-keygen new --outfile backend-authority.json --no-bip39-passphrase
```

### 3. Program Configuration

The program ID is automatically set after deployment. Update `backend/src/merkle/solana_client.rs`:

```rust
const PROGRAM_ID: &str = "YOUR_PROGRAM_ID_HERE";
```

## 🎮 Usage

### 1. Start Local Validator

```bash
# Terminal 1
cd merkle-program
solana-test-validator
```

### 2. Deploy Program

```bash
# Terminal 2
cd merkle-program
anchor build
anchor deploy

# Note the Program ID and update backend/src/merkle/solana_client.rs
solana-keygen pubkey target/deploy/merkle_program-keypair.json
```

### 3. Fund Authority Wallet

```bash
cd backend
solana airdrop 2 $(solana-keygen pubkey ./backend-authority.json) --url http://localhost:8899
```

### 4. Generate Test Data

```bash
cd backend
cargo run
```

The backend will:

1. ✅ Connect to database
2. ✅ Check/initialize Solana program config
3. ✅ Build merkle tree from subscribers
4. ✅ Sync root hash to Solana
5. ✅ Verify proofs work correctly

### Expected Output

```
✅ Successfully connected to database!
✅ Connected to Solana RPC: http://localhost:8899

🔍 Checking program config...
   ✅ Config account exists
   Current root: 1f7cd27e7a04...

🌲 Merkle Tree Built:
   Root Hash: 1f7cd27e7a04eabb9a707a3ff56055f4e2e2cdd7958a6236e532006fa434a623
   Total subscribers: 10

📤 Syncing merkle root to Solana...
✅ Updated merkle root on-chain
   New Root: 1f7cd27e7a04...
   Signature: jNCWRWYHTMQ...
✅ Successfully updated on-chain!

🔐 Testing Proof Verification...
   User: 5sHXVAK46po96V9syX6Jhmav9qUagnKydnYwuE57KPat
   Expiration: 1769665096
   Off-chain verification: ✓ VALID

🧪 Testing Tampering Detection...
   Tampered expiration: ✓ REJECTED (Correct)
```

## 🧪 Testing

### Run Backend Tests

```bash
cd backend
cargo test
```

### Run Anchor Tests

```bash
cd merkle-program
anchor test --skip-local-validator
```

## 🎓 How It Works: Deep Dive

### Leaf Construction

Each subscriber becomes a leaf in the tree:

```rust
// Leaf = SHA256(pubkey_bytes[32] + expiration_i64[8])
let mut leaf_data = Vec::with_capacity(40);
leaf_data.extend_from_slice(&pubkey_bytes);      // 32 bytes
leaf_data.extend_from_slice(&expiration.to_le_bytes()); // 8 bytes
let leaf = SHA256(leaf_data); // 32 bytes
```

### Proof Generation

When a user requests proof:

1. Find their leaf index in sorted tree
2. Generate sibling hashes for path to root
3. Serialize proof (typically 10-15 hashes for 10k users)
4. Return proof + index + expiration

### On-Chain Verification

```rust
// 1. Reconstruct leaf
let leaf = Hash(user_pubkey + expiration);

// 2. Walk up the tree using proof
for sibling in proof {
    current = Hash(min(current, sibling) + max(current, sibling));
}

// 3. Compare with stored root
require!(current == config.merkle_root);
```

### Security Guarantees

- **Immutability**: Cannot fake membership without private key
- **Integrity**: Any data tampering changes the root hash
- **Expiration**: On-chain clock prevents expired access
- **Authority**: Only backend can update root

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

**Built with ❤️ using Rust, Solana, and Anchor**
