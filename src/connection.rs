use super::*;

use crate::routing::Context;
use ibc::{
	clients::ics10_grandpa::{consensus_state::ConsensusState as GPConsensusState, header::Header},
	core::{
		ics02_client::{client_consensus::AnyConsensusState, client_state::AnyClientState},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as Ics03Error,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::{ClientId, ConnectionId},
	},
	Height,
};

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
		log::trace!("in connection : [connection_end]");

		if <Connections<T>>::contains_key(conn_id.as_bytes()) {
			let data = <Connections<T>>::get(conn_id.as_bytes());
			let ret =
				ConnectionEnd::decode_vec(&*data).map_err(|e| Ics03Error::invalid_decode(e))?;
			log::trace!("in connection : [connection_end] >>  connection_end = {:?}", ret);
			Ok(ret)
		} else {
			log::trace!("in connection : [connection_end] >> read connection end returns None");
			Err(Ics03Error::connection_mismatch(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics03Error> {
		log::trace!("in connection : [client_state]");

		// ClientReader::client_state(self, client_id)
		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			let result =
				AnyClientState::decode_vec(&*data).map_err(|e| Ics03Error::invalid_decode(e))?;
			log::trace!("in connection : [client_state] >> client_state: {:?}", result);
			Ok(result)
		} else {
			log::trace!("in connection : [client_state] >> read client_state is None");

			Err(Ics03Error::frozen_client(client_id.clone()))
		}
	}

	fn host_current_height(&self) -> Height {
		log::trace!("in connection : [host_current_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();

		<OldHeight<T>>::put(current_height);

		log::trace!(
			"in connection : [host_current_height] >> Host current height = {:?}",
			Height::new(0, current_height)
		);
		Height::new(0, current_height)
	}

	fn host_oldest_height(&self) -> Height {
		log::trace!("in connection : [host_oldest_height]");

		let height = <OldHeight<T>>::get();

		log::trace!(
			"in connection : [host_oldest_height] >> Host oldest height = {:?}",
			Height::new(0, height)
		);
		Height::new(0, height)
	}

	fn connection_counter(&self) -> Result<u64, Ics03Error> {
		log::trace!("in connection : [connection_counter]");

		Ok(<ConnectionCounter<T>>::get())
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		log::trace!("in connection : [commitment_prefix]");

		"ibc".as_bytes().to_vec().try_into().unwrap_or_default()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics03Error> {
		log::trace!("in connection : [client_consensus_state]");

		// ClientReader::consensus_state(self, client_id, height)
		let height = height.encode_vec().map_err(|e| Ics03Error::invalid_encode(e))?;
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1)
					.map_err(|e| Ics03Error::invalid_decode(e))?;
				return Ok(any_consensus_state)
			}
		}

		// Err(ICS03Error::missing_consensus_height())
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics03Error> {
		log::trace!("in connection : [host_consensus_state]");
		let result = AnyConsensusState::Grandpa(GPConsensusState::from(Header::default()));

		log::trace!("in connection : [host_consensus_state] >> any_consensus_state = {:?}", result);
		Ok(result)
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn increase_connection_counter(&mut self) {
		log::trace!("in connection : [increase_connection_counter]");

		let ret = <ConnectionCounter<T>>::try_mutate(|val| -> Result<(), Ics03Error> {
			let new =
				val.checked_add(1).ok_or(Ics03Error::invalid_increment_connection_counter())?;
			*val = new;
			Ok(())
		});
	}

	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), Ics03Error> {
		log::trace!("in connection : [store_connection]");

		let data = connection_end.encode_vec().map_err(|e| Ics03Error::invalid_encode(e))?;

		// store connection end
		<Connections<T>>::insert(connection_id.as_bytes().to_vec(), data);

		// store connection id vector for rpc
		let ret = <ConnectionsKeys<T>>::try_mutate(|val| -> Result<(), Ics03Error> {
			if let Some(_value) = val.iter().find(|&x| x == connection_id.as_bytes()) {
			} else {
				val.push(connection_id.as_bytes().to_vec());
			}
			Ok(())
		});

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), Ics03Error> {
		log::trace!("in connection : [store_connection_to_client]");

		<ConnectionClient<T>>::insert(
			client_id.as_bytes().to_vec(),
			connection_id.as_bytes().to_vec(),
		);
		Ok(())
	}
}
