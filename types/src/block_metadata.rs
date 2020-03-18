// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::association_address, event::EventKey,
};
use anyhow::Result;
use libra_crypto::{ed25519::Ed25519Signature, HashValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Struct that will be persisted on chain to store the information of the current block.
///
/// The flow will look like following:
/// 1. The executor will pass this struct to VM at the end of a block proposal.
/// 2. The VM will use this struct to create a special system transaction that will modify the on
///    chain resource that represents the information of the current block. This transaction can't
///    be emitted by regular users and is generated by each of the validators on the fly. Such
///    transaction will be executed before all of the user-submitted transactions in the blocks.
/// 3. Once that special resource is modified, the other user transactions can read the consensus
///    info by calling into the read method of that resource, which would thus give users the
///    information such as the current leader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    id: HashValue,
    timestamp_usecs: u64,
    // Since Move doesn't support hashmaps, this vote map would be stored as a vector of key value
    // pairs in the Move module. Thus we need a BTreeMap here to define how the values are being
    // ordered.
    previous_block_votes: BTreeMap<AccountAddress, Ed25519Signature>,
    proposer: AccountAddress,
}

impl BlockMetadata {
    pub fn new(
        id: HashValue,
        timestamp_usecs: u64,
        previous_block_votes: BTreeMap<AccountAddress, Ed25519Signature>,
        proposer: AccountAddress,
    ) -> Self {
        Self {
            id,
            timestamp_usecs,
            previous_block_votes,
            proposer,
        }
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn into_inner(self) -> Result<(Vec<u8>, u64, Vec<u8>, AccountAddress)> {
        let id = self.id.to_vec();
        let vote_maps = lcs::to_bytes(&self.previous_block_votes)?;
        Ok((id, self.timestamp_usecs, vote_maps, self.proposer))
    }

    pub fn proposer(&self) -> AccountAddress {
        self.proposer
    }

    pub fn voters(&self) -> Vec<AccountAddress> {
        self.previous_block_votes.keys().cloned().collect()
    }
}

pub fn new_block_event_key() -> EventKey {
    EventKey::new_from_address(&association_address(), 2)
}
