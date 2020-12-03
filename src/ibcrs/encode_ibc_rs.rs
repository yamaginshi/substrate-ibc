use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
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
    pub trust_level: Vec<u8>,
    pub trusting_period: u64,
    pub unbonding_period: u64,
    pub max_clock_drift: u64,
    pub frozen_height: Height,
    pub latest_height: Height,
    pub consensus_params: Vec<u8>,
    pub upgrade_path: Vec<u8>,
    pub allow_update_after_expiry: bool,
    pub allow_update_after_misbehaviour: bool,
}

