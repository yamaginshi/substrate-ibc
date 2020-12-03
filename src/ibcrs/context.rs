use ibc::ics02_client::context::ClientReader;
use ibc::ics02_client::client_type::ClientType;
use ibc::ics02_client::client_def::{AnyClientState, AnyConsensusState};
use ibc::Height;
use ibc::ics24_host::identifier::{ChainId, ClientId, ConnectionId};
use ibc::mock::host::{HostBlock, HostType};
use ibc::mock::client_state::MockClientRecord;
use ibc::ics03_connection::connection::ConnectionEnd;

use std::cmp::min;
use std::collections::HashMap;

pub struct IbcContext {
    /// The type of host chain underlying this mock context.
    host_chain_type: HostType,

    /// Host chain identifier.
    host_chain_id: ChainId,

    /// Maximum size for the history of the host chain. Any block older than this is pruned.
    max_history_size: usize,

    /// Highest height (i.e., most recent) of the blocks in the history.
    latest_height: Height,

    /// The chain of blocks underlying this context. A vector of size up to `max_history_size`
    /// blocks, ascending order by their height (latest block is on the last position).
    history: Vec<HostBlock>,

    /// The set of all clients, indexed by their id.
    // clients: HashMap<ClientId, MockClientRecord>,

    /// Association between client ids and connection ids.
    client_connections: HashMap<ClientId, ConnectionId>,

    /// All the connections in the store.
    connections: HashMap<ConnectionId, ConnectionEnd>,
}

impl Default for IbcContext {
    fn default() -> Self {
        Self::new(
            ChainId::new("ibc_gaia".to_string(), 1),
            HostType::SyntheticTendermint,
            5,
            Height::new(1, 5),
        )
    }
}

impl ClientReader for IbcContext {
    fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
        // match self.clients.get(client_id) {
        //     Some(client_record) => client_record.client_type.into(),
        //     None => None,
        // }
        None
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        // match self.clients.get(client_id) {
        //     Some(client_record) => client_record.client_state.clone(),
        //     None => None,
        // }
        None
    }

    fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
        // match self.clients.get(client_id) {
        //     Some(client_record) => match client_record.consensus_states.get(&height) {
        //         Some(consensus_state) => Option::from(consensus_state.clone()),
        //         None => None,
        //     },
        //     None => None,
        // }
        None
    }
}

impl IbcContext {
    /// Creates a mock context. Parameter `max_history_size` determines how many blocks will
    /// the chain maintain in its history, which also determines the pruning window. Parameter
    /// `latest_height` determines the current height of the chain. This context
    /// has support to emulate two type of underlying chains: Mock or SyntheticTendermint.
    pub fn new(
        host_id: ChainId,
        host_type: HostType,
        max_history_size: usize,
        latest_height: Height,
    ) -> Self {
        assert_ne!(
            max_history_size, 0,
            "The chain must have a non-zero max_history_size"
        );

        // Compute the number of blocks to store. If latest_height is 0, nothing is stored.
        let n = min(max_history_size as u64, latest_height.version_height);

        assert_eq!(
            ChainId::chain_version(&host_id.to_string()),
            latest_height.version_number,
            "The version in the chain identifier must match the version in the latest height"
        );

        IbcContext {
            host_chain_type: host_type,
            host_chain_id: host_id.clone(),
            max_history_size,
            latest_height,
            history: (0..n)
                .rev()
                .map(|i| {
                    HostBlock::generate_block(
                        host_id.clone(),
                        host_type,
                        latest_height.sub(i).unwrap().version_height,
                    )
                })
                .collect(),
            connections: Default::default(),
            // clients: Default::default(),
            client_connections: Default::default(),
        }
    }
}