#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;


use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, B256, ruint::aliases::U64};
use alloy_sol_types::sol;
use stylus_sdk::{prelude::*, storage::*, stylus_core::errors::MethodError};



//Contract Address on Sepolia : 0x468edc5b2fe9d1c919f2377cbe0ccb16f32ead29
//verify contract on sepolia using https://sepolia.arbiscan.io/address/0x468edc5b2fe9d1c919f2377cbe0ccb16f32ead29
//contract transaction hash : 0x7ec3ca01306a0218364f9715c20733b1f7208190b3bd866ac7739900216f05cf

sol! {
    event ContentRegistered(
        bytes32 indexed sha256hash,
        address indexed creator,
        uint64 phash,
        uint64 timestamp,
        string ipfsCid,
        string aitool
    );
    error ContentAlreadyRegistered(bytes32 sha256hash);
    error ContentNotFound(bytes32 sha256hash);
}

#[storage]
pub struct ContentRecord {
    pub creator: StorageAddress,
    pub timestamp: StorageU64,
    pub phash: StorageU64,
    pub ipfs_cid: StorageString,
    pub ai_tool: StorageString,
    pub is_registered: StorageBool,
}

#[storage]
#[entrypoint]
pub struct VeritraceRegistry {
    pub registry: StorageMap<B256, ContentRecord>,
}

#[public]
impl VeritraceRegistry {
    pub fn register_content(
        &mut self,
        sha256hash: B256,
        phash: u64,
        ipfs_cid: String,
        ai_tool: String,
    ) -> Result<(), Vec<u8>> {
        let creator = self.vm().msg_sender();
        let timestamp = self.vm().block_timestamp();
        let mut record = self.registry.setter(sha256hash);

        if record.is_registered.get() {
            return Err(ContentAlreadyRegistered {
                sha256hash: sha256hash.into(),
            }
            .encode());
        }


        record.creator.set(creator);
        record.timestamp.set(U64::from(timestamp));
        record.phash.set(U64::from(phash));
        record.ipfs_cid.set_str(&ipfs_cid);
        record.ai_tool.set_str(&ai_tool);
        record.is_registered.set(true);

        self.vm().log(ContentRegistered {
            sha256hash: sha256hash.into(),
            creator,
            phash,
            timestamp,
            ipfsCid: ipfs_cid,
            aitool: ai_tool,
        });

        Ok(())
    }
    pub fn verify_content(
        &self,
        sha256hash: B256,
    ) -> Result<(Address, u64, u64, String, String), Vec<u8>> {
        let record = self.registry.getter(sha256hash);

        if !record.is_registered.get() {
            return Err(ContentNotFound {
                sha256hash: sha256hash.into(),
            }
            .encode());
        }

        Ok((
            record.creator.get(),
            record.timestamp.get().to::<u64>(),
            record.phash.get().to::<u64>(),
            record.ipfs_cid.get_string(),
            record.ai_tool.get_string(),
        ))
    }
}
