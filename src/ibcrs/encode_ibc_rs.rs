use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use std::time::Duration;
use sp_core::H256;

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ConsensusState {
    pub timestamp: u128,  // Timestamp in utc time zone
    pub root: Vec<u8>,
    pub next_validators_hash: [u8; 32],
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct TrustThresholdFraction {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct Height {
    pub version_number: u64,
    pub version_height: u64,
}

// Converted from use crate::ics07_tendermint::client_state::ClientState;
#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ClientState {
    pub chain_id: H256,
    pub trust_level: TrustThresholdFraction,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Height,
    pub latest_height: Height,
    // pub consensus_params: Params,
    // pub upgrade_path: String,
    // pub allow_update_after_expiry: bool,
    // pub allow_update_after_misbehaviour: bool,
}

