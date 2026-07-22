# VeriTrace Smart Contract Registry

The immutable truth and provenance layer for the VeriTrace ecosystem. Built using the high-performance **Arbitrum Stylus Rust SDK**, this contract acts as a decentralized registry for cryptographic media fingerprints, structural perceptual hashes, and storage content IDs (CIDs).

---

## Deployment Information (Arbitrum Sepolia)

The contract has been successfully compiled into optimized WebAssembly (Wasm), deployed, and activated within the ArbOS state.

| Parameter | Value |
| :--- | :--- |
| **Contract Address** | ` 0xeb09ca3b844693817479cf33fd88cdf02c2711fd`|
| **Deployment Tx Hash** | `0x4ed05785a8f74f889ab1b87bf98222cc272124890ca78a7e1138685b69c88992` |
| **Network** | Arbitrum Sepolia Testnet |
| **Explorer Link** | [View on Arbiscan](https://sepolia.arbiscan.io/address/0xd5a4e9185cbcea881f2c76b07732335250537820) |

---

##  Architecture & Storage Strategy


1. **On-Chain Anchor:** Stores a unique 32-byte cryptographic identifier (`SHA-256`), its visual structural fingerprint (`pHash`), the generating engine attribution (`ai_tool`), and the creator signature.
2. **Off-Chain Media Layer:** The heavy images and videos are pushed directly to decentralized storage (**IPFS**). The resulting permanent `ipfs_cid` is recorded on-chain inside the structural mapping for decentralized lookup.

### Storage Layout Struct

```rust
pub struct ContentRecord {
    pub creator: StorageAddress,       // Public key address of the initial registrar
    pub timestamp: StorageU64,         // Block production time of proof commitment
    pub phash: StorageU64,             // 64-bit integer representation of visual structure
    pub ipfs_cid: StorageString,       // Decoupled decentralized media pointer 
    pub ai_tool: StorageString,        // Attribution identification tag (e.g., "DALL-E 3")
    pub is_registered: StorageBool,    // Verification safety flag
}
