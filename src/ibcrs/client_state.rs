use std::collections::HashMap;
// use std::convert::{TryFrom, TryInto};
//
// use tendermint_proto::Protobuf;
//
// use ibc::ics07_tendermint::client_state::ClientState as RawMockClientState;
// use ibc::ics02_client::client_def::AnyConsensusState as RawMockConsensusState;
//
use ibc::ics02_client::client_def::{AnyClientState, AnyConsensusState};
use ibc::ics02_client::client_type::ClientType;
// use ibc::ics02_client::error::Error;
// use ibc::ics02_client::error::Kind;
// use ibc::ics02_client::state::{ClientState, ConsensusState};
// use ibc::ics23_commitment::commitment::CommitmentRoot;
// use ibc::mock::header::MockHeader;
use ibc::Height;

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The type of this client.
    pub client_type: ClientType,

    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<AnyClientState>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: HashMap<Height, AnyConsensusState>,
}
