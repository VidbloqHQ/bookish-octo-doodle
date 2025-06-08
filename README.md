
```
# 🎥 VidBloq Program

A custom [Anchor](https://book.anchor-lang.com/) smart contract that powers stream-specific wallets on the Solana blockchain. This program enables users to create and manage on-chain wallets for live streams or events, allowing token deposits, distributions, and refunds—enabling use cases like real-time tipping, gated access, or DAO rewards.

## ✨ Features

- ✅ Create program-derived wallets for individual streams
- 💸 Allow users to deposit SPL tokens into stream wallets
- 🪙 Distribute tokens to participants (e.g. hosts, guests)
- 🔁 Refund remaining tokens to donors after a stream ends
- 🧱 Built with Anchor + Solana best practices
- 🔐 Secure account validation and ownership enforcement

## 📦 Repo Structure

├── programs/
│   └── stream-wallet/     # Anchor smart contract
├── tests/                 # Mocha + Anchor tests
├── migrations/            # Anchor deployment scripts
└── README.md

```

## 🚀 Getting Started

### Prerequisites

- Node.js ≥ 16.x
- Anchor CLI ≥ 0.29.x
- Solana CLI ≥ 1.10.x
- Rust ≥ 1.60+

### 1. Install Dependencies
npm install

### 2. Build the Program

```
anchor build
```

### 3. Deploy to Localnet

```
anchor localnet
```

### 4. Run Tests

```
anchor test
```

> Ensure you have a local validator running (e.g. via `anchor localnet`).

## 🧠 Program Overview

| Instruction    | Description                                          |
| -------------- | ---------------------------------------------------- |
| `createStream` | Initializes a stream wallet (PDA) with metadata      |
| `deposit`      | Allows users to deposit SPL tokens into the stream   |
| `distribute`   | Distributes tokens from the wallet to participant(s) |
| `refund`       | Refunds unused tokens to original donors             |

All actions are authorized via signer checks or PDAs, ensuring trustless handling.

## 🛠 Tech Stack

* [Solana](https://solana.com/)
* [Anchor](https://book.anchor-lang.com/)
* [TypeScript](https://www.typescriptlang.org/) (for tests + SDK)
* [SPL Token](https://spl.solana.com/token)

## 🌐 SDK & Frontend

The companion SDK is available here: https://github.com/VidbloqHQ/studious-robot

Demo: https://jade-duckanoo-edb1bd.netlify.app/

> The SDK provides a simple React hook for integrating the program into your dApp. Frontend UI is currently minimal and under development.

## 🧩 Future Plans

* Support for SOL in addition to SPL tokens
* NFT and cNFT-based stream gating
* DAO treasury integration for rewards
* Full-featured frontend dashboard

## 🧑‍💻 Author

**Chiamaka Ezemba** – https://x.com/Ada_ezemba
Solana developer passionate about real-time experiences, protocol design, and DevX tooling.
