use std::marker::PhantomData;
use std::sync::Arc;

use ethers::types::U256;
// Substrate
use scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;
// Starknet
use starknet_api::block::BlockHash;
use starknet_api::hash::StarkFelt;
use uuid::Uuid;

use crate::DbHash;

// The fact db stores DA facts that need to be written to L1
pub struct DaDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

// TODO: purge old cairo job keys
impl<B: BlockT> DaDb<B> {
    pub fn state_diff(&self, block_hash: &BlockHash) -> Result<Vec<U256>, String> {
        match self.db.get(crate::columns::DA, block_hash.0.bytes()) {
            Some(raw) => Ok(Vec::<U256>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Err(String::from("can't write state diff")),
        }
    }

    pub fn store_state_diff(&self, block_hash: &BlockHash, diff: Vec<U256>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, block_hash.0.bytes(), &diff.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn cairo_job(&self, block_hash: &BlockHash) -> Result<Uuid, String> {
        match self.db.get(crate::columns::DA, block_hash.0.bytes()) {
            Some(raw) => Ok(Uuid::from_slice(&raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Err(String::from("can't locate cairo job")),
        }
    }

    pub fn update_cairo_job(&self, block_hash: &BlockHash, job_id: Uuid) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, block_hash.0.bytes(), &job_id.into_bytes());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn last_proved_block(&self) -> Result<BlockHash, String> {
        match self.db.get(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK) {
            Some(raw) => {
                let felt = StarkFelt::deserialize(&raw[..]).ok_or("Failed to deserialize block hash")?;
                Ok(BlockHash(felt))
            }
            None => Err(String::from("can't locate last proved block")),
        }
    }

    pub fn update_last_proved_block(&self, block_hash: &BlockHash) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK, block_hash.0.bytes());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}