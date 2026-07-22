#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256, ruint::aliases::U64};
use alloy_sol_types::sol;
use stylus_sdk::{prelude::*, storage::*, stylus_core::errors::MethodError};



//Contract Address :  0xeb09ca3b844693817479cf33fd88cdf02c2711fd
// Contract Transactions hash : 0x4ed05785a8f74f889ab1b87bf98222cc272124890ca78a7e1138685b69c88992


sol_interface! {
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
    }
}

sol! {
    event ContentRegistered(
        bytes32 indexed sha256hash,
        address indexed creator,
        uint64 phash,
        uint64 timestamp,
        string ipfsCid,
        string aitool,
        bool allowAiTraining
    );
    event DatasetPurchased(
        address indexed buyer,
        uint256 totalUsdc
    );
    error ContentAlreadyRegistered(bytes32 sha256hash);
    error ContentNotFound(bytes32 sha256hash);
    error TransferFailed();
}

#[storage]
pub struct ContentRecord {
    pub creator: StorageAddress,
    pub timestamp: StorageU64,
    pub phash: StorageU64,
    pub ipfs_cid: StorageString,
    pub ai_tool: StorageString,
    pub is_registered: StorageBool,
    pub allow_ai_training: StorageBool,
}

#[storage]
#[entrypoint]
pub struct VeritraceRegistry {
    pub registry: StorageMap<B256, ContentRecord>,
    pub owner: StorageAddress,
    pub initialized: StorageBool,
    pub verified_publishers: StorageMap<Address, StorageString>,
}

#[public]
impl VeritraceRegistry {
    pub fn initialize(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }
        self.owner.set(owner);
        self.initialized.set(true);
        Ok(())
    }

    pub fn register_content(
        &mut self,
        sha256hash: B256,
        phash: u64,
        ipfs_cid: String,
        ai_tool: String,
        allow_ai_training: bool,
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
        record.allow_ai_training.set(allow_ai_training);

        self.vm().log(ContentRegistered {
            sha256hash: sha256hash.into(),
            creator,
            phash,
            timestamp,
            ipfsCid: ipfs_cid,
            aitool: ai_tool,
            allowAiTraining: allow_ai_training,
        });

        Ok(())
    }

    pub fn verify_content(
        &self,
        sha256hash: B256,
    ) -> Result<(Address, u64, u64, String, String, bool), Vec<u8>> {
        let record = self.registry.getter(sha256hash);

        if !record.is_registered.get() {
            return Err(ContentNotFound {
                sha256hash: sha256hash.into(),
            }.encode());
        }

        Ok((
            record.creator.get(),
            record.timestamp.get().to::<u64>(),
            record.phash.get().to::<u64>(),
            record.ipfs_cid.get_string(),
            record.ai_tool.get_string(),
            record.allow_ai_training.get(),
        ))
    }

    pub fn purchase_dataset_access(
        &mut self,
        token: Address,
        creators: Vec<Address>,
        amounts: Vec<U256>,
        total_usdc: U256,
    ) -> Result<(), Vec<u8>> {
        if creators.len() != amounts.len() {
            return Err(b"Array lengths must match".to_vec());
        }

        let buyer = self.vm().msg_sender();
        let this_contract = self.vm().contract_address();

        let erc20 = IERC20::new(token);

        let config = Call::new_mutating(self);
        let success = erc20
            .transfer_from(self.vm(), config, buyer, this_contract, total_usdc)
            .map_err(|_| TransferFailed {}.encode())?;

        if !success {
            return Err(TransferFailed {}.encode());
        }

        let mut total_paid = U256::ZERO;
        for i in 0..creators.len() {
            let creator = creators[i];
            let amount = amounts[i];

            if amount > U256::ZERO {
                // Must create a new config for each call since config is consumed
                let transfer_config = Call::new_mutating(self);
                let transfer_success = erc20
                    .transfer(self.vm(), transfer_config, creator, amount)
                    .map_err(|_| TransferFailed {}.encode())?;

                if !transfer_success {
                    return Err(TransferFailed {}.encode());
                }
                total_paid += amount;
            }
        }

        if total_paid > total_usdc {
            return Err(b"Paid out more than total".to_vec());
        }

        self.vm().log(DatasetPurchased {
            buyer,
            totalUsdc: total_usdc,
        });

        Ok(())
    }

    pub fn withdraw_treasury(&mut self, token: Address) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let owner = self.owner.get();
        if sender != owner {
            return Err(b"Not authorized".to_vec());
        }

        let this_contract = self.vm().contract_address();

        let erc20 = IERC20::new(token);

        let balance_config = Call::new();
        let balance = erc20
            .balance_of(self.vm(), balance_config, this_contract)
            .map_err(|_| b"BalanceOf call failed".to_vec())?;

        if balance > U256::ZERO {
            let transfer_config = Call::new_mutating(self);
            let success = erc20
                .transfer(self.vm(), transfer_config, owner, balance)
                .map_err(|_| TransferFailed {}.encode())?;

            if !success {
                return Err(TransferFailed {}.encode());
            }
        }

        Ok(())
    }

    pub fn add_verified_publisher(
        &mut self,
        publisher: Address,
        org_name: String,
    ) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let owner = self.owner.get();
        if sender != owner {
            return Err(b"Not authorized".to_vec());
        }
        let mut name_ref = self.verified_publishers.setter(publisher);
        name_ref.set_str(&org_name);
        Ok(())
    }

    pub fn remove_verified_publisher(
        &mut self,
        publisher: Address,
    ) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let owner = self.owner.get();
        if sender != owner {
            return Err(b"Not authorized".to_vec());
        }
        let mut name_ref = self.verified_publishers.setter(publisher);
        name_ref.set_str("");
        Ok(())
    }

    pub fn is_verified_publisher(
        &self,
        publisher: Address,
    ) -> Result<(String, bool), Vec<u8>> {
        let name_ref = self.verified_publishers.getter(publisher);
        let name = name_ref.get_string();
        if name.is_empty() {
            return Ok((name, false));
        }
        Ok((name, true))
    }
}
