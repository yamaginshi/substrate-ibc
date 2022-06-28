use crate::{
	utils::{host_height, LOG_TARGET},
	*,
};
use log::trace;
use scale_info::prelude::{
	format,
	string::{String, ToString},
};
use sp_std::str::FromStr;

use crate::context::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as ICS02Error,
		},
		ics24_host::identifier::ClientId,
	},
	timestamp::Timestamp,
	Height,
};

/// Defines the read-only part of ICS02 (client functions) context.
impl<T: Config> ClientReader for Context<T> {
	/// Read `ClientType` by `ClientId`.
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ICS02Error> {
		trace!(target: LOG_TARGET, "in client : [client_type] >> client_id = {:?}", client_id);

		let encode_client_id = client_id.as_bytes();

		match <Clients<T>>::contains_key(encode_client_id) {
			true => {
				let encode_client_type = <Clients<T>>::get(encode_client_id);
				let string_client_type =
					String::from_utf8(encode_client_type).map_err(ICS02Error::invalid_from_utf8)?;
				let client_type = ClientType::from_str(&string_client_type)
					.map_err(|e| ICS02Error::unknown_client_type(e.to_string()))?;

				trace!(
					target: LOG_TARGET,
					"in client : [client_type] >> client_type = {:?}",
					client_type
				);
				Ok(client_type)
			},
			false => {
				error!(
					target: LOG_TARGET,
					"in client : [client_type] :❎ Can't ClientType by ClientId({})", client_id
				);
				Err(ICS02Error::client_not_found(client_id.clone()))
			},
		}
	}

	/// Read `AnyClientState` by `ClientId`.
	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS02Error> {
		trace!(target: LOG_TARGET, "in client : [client_state] >> client_id = {:?}", client_id);

		match <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			true => {
				let encode_client_state = <ClientStates<T>>::get(client_id.as_bytes());
				let any_client_state = AnyClientState::decode_vec(&*encode_client_state)
					.map_err(ICS02Error::invalid_decode)?;
				trace!(
					target: LOG_TARGET,
					"in client : [client_state] >> any client_state: {:?}",
					any_client_state
				);

				Ok(any_client_state)
			},
			false => {
				error!(
					target: LOG_TARGET,
					"in client : [client_state] ❎ Can't ClientState by ClientId({})", client_id
				);
				Err(ICS02Error::client_not_found(client_id.clone()))
			},
		}
	}

	/// Retrieve the consensus state for give client ID at the specified height.
	///
	/// Returns an error if no such stat exists.
	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_client_id = client_id.as_bytes();
		let encode_height = height.encode_vec().map_err(ICS02Error::invalid_decode)?;
		let encode_consensus_state = <ConsensusStates<T>>::get(encode_client_id, encode_height);

		let any_consensus_state = AnyConsensusState::decode_vec(&*encode_consensus_state)
			.map_err(|_| ICS02Error::consensus_state_not_found(client_id.clone(), height))?;

		trace!(
			target: LOG_TARGET,
			"in client: [consensus_state] >> any consensus state = {:?}",
			any_consensus_state
		);
		Ok(any_consensus_state)
	}

	/// Search for the lowest consensus state higher than `height`.
	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [next_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_client_id = client_id.as_bytes();
		let mut cs_with_height = <ConsensusStates<T>>::iter_key_prefix(encode_client_id.clone())
			.map(|height| {
				let cs_state = <ConsensusStates<T>>::get(encode_client_id, height.clone());
				let height = Height::decode_vec(&height).map_err(ICS02Error::invalid_decode)?;
				let any_cs =
					AnyConsensusState::decode_vec(&cs_state).map_err(ICS02Error::invalid_decode)?;

				Ok((height, any_cs))
			})
			.collect::<Result<Vec<_>, ICS02Error>>()?;

		cs_with_height.sort_by(|a, b| a.0.cmp(&b.0));

		for cs in cs_with_height.into_iter() {
			if cs.0 > height {
				return Ok(Some(cs.1))
			}
		}

		Ok(None)
	}

	/// Search for the highest consensus state lower than `height`.
	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [next_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_client_id = client_id.as_bytes();
		let mut cs_with_height = <ConsensusStates<T>>::iter_key_prefix(encode_client_id.clone())
			.map(|height| {
				let cs_state = <ConsensusStates<T>>::get(encode_client_id, height.clone());
				let height = Height::decode_vec(&height).map_err(ICS02Error::invalid_decode)?;
				let any_cs =
					AnyConsensusState::decode_vec(&cs_state).map_err(ICS02Error::invalid_decode)?;

				Ok((height, any_cs))
			})
			.collect::<Result<Vec<_>, ICS02Error>>()?;

		cs_with_height.sort_by(|a, b| a.0.cmp(&b.0));

		for cs in cs_with_height.into_iter() {
			if cs.0 < height {
				return Ok(Some(cs.1))
			}
		}

		Ok(None)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		trace!(target: LOG_TARGET, "in client : [host_height]");

		let revision_height = host_height::<T>();
		trace!(
			target: LOG_TARGET,
			"in channel: [host_height] >> revision_height = {:?}",
			revision_height
		);
		let revision_number = 0; // todo revision_number is zero.
		Height::new(revision_number, revision_height)
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		trace!(target: LOG_TARGET, "in client: [host_timestamp]");

		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		trace!(
			target: LOG_TARGET,
			"in client: [host_timestamp] >> host_timestamp = {:?}",
			ts.unwrap()
		);

		ts.unwrap()
	}

	/// Returns the `ConsensusState` of the host (local) chain at specific height.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, ICS02Error> {
		trace!(target: LOG_TARGET, "in client : [consensus_state] >> height = {:?}", height);

		let bounded_map = <HostConsensusState<T>>::get();
		let local_consensus_state = bounded_map.get(&height.revision_height).expect(&format!(
			"[host_consensus_state]: consensus state not found for host at height {}",
			height
		));
		let timestamp = Timestamp::from_nanoseconds(local_consensus_state.timestamp)
			.expect("[host_consensus_state]: error decoding timestamp")
			.into_tm_time()
			.expect("[host_consensus_state]: Could not convert timestamp into tendermint time");
		let consensus_state = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			timestamp,
			root: local_consensus_state.commitment_root.clone().into(),
		};
		Ok(AnyConsensusState::Grandpa(consensus_state))
	}

	/// Returns the pending `ConsensusState` of the host (local) chain.
	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ICS02Error> {
		trace!(target: LOG_TARGET, "in client: [pending_host_consensus_state]");

		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	/// Returns a natural number, counting how many clients have been created thus far.
	/// The value of this counter should increase only via method
	/// `ClientKepper::increase_client_counter`.
	fn client_counter(&self) -> Result<u64, ICS02Error> {
		trace!(target: LOG_TARGET, "in client : [client_counter]");

		Ok(<ClientCounter<T>>::get())
	}
}

/// Defines the write-only part of ICS02 (client functions) context.
impl<T: Config> ClientKeeper for Context<T> {
	/// Called upon successful client creation.
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [store_client_type] >> client_id = {:?}, client_type = {:?}",
			client_id,
			client_type
		);

		let encode_client_id = client_id.as_bytes();
		let encode_client_type = client_type.as_str().as_bytes();

		<Clients<T>>::insert(encode_client_id, encode_client_type);

		Ok(())
	}

	/// Called upon successful client creation and update
	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [store_client_state] >> client_id = {:?}, client_state = {:?}",
			client_id,
			client_state
		);

		let encode_client_type = client_id.as_bytes();
		let encode_client_state = client_state.encode_vec().map_err(ICS02Error::invalid_encode)?;

		<ClientStates<T>>::insert(encode_client_type, encode_client_state);

		Ok(())
	}

	/// Called upon successful client creation and update.
	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client : [store_consensus_state] \
			>> client_id = {:?}, height = {:?}, consensus_state = {:?}",
			client_id,
			height,
			consensus_state
		);

		let encode_client_type = client_id.as_bytes();
		let encode_height = height.encode_vec().map_err(ICS02Error::invalid_encode)?;
		let encode_consensus_state =
			consensus_state.encode_vec().map_err(ICS02Error::invalid_encode)?;

		<ConsensusStates<T>>::insert(encode_client_type, encode_height, encode_consensus_state);

		Ok(())
	}

	/// Called upon client creation.
	/// Increases the counter which keeps track of how many clients have been created.
	/// Should never fail.
	fn increase_client_counter(&mut self) {
		trace!(target: LOG_TARGET, "in client : [increase_client_counter]");

		<ClientCounter<T>>::try_mutate(|val| -> Result<(), ICS02Error> {
			let new = val.checked_add(1).ok_or_else(ICS02Error::invalid_increase_client_counter)?;
			*val = new;
			Ok(())
		})
		.expect("increase_client_counter error");
	}

	/// Called upon successful client update.
	/// Implementations are expected to use this to record specified time
	/// as the time at which this update (or header) was processed.
	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client: [store_update_time] >>\
			 client_id = {:?}, height = {:?}, timestamp = {:?}",
			client_id,
			height,
			timestamp
		);

		let encode_client_id = client_id.as_bytes();
		let encode_height = height.encode_vec().map_err(ICS02Error::invalid_encode)?;
		let timestamp = timestamp.nanoseconds();

		<ClientUpdateTime<T>>::insert(encode_client_id, encode_height, timestamp);

		Ok(())
	}

	/// Called upon successful client update.
	/// Implementations are expected to use this to record the specified
	/// height as the height at which this update (or header) was processed.
	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ICS02Error> {
		trace!(
			target: LOG_TARGET,
			"in client: [store_update_height] >>\
			 client_id = {:?}, height = {:?}, host_height = {:?}",
			client_id,
			height,
			host_height
		);

		let encode_client_id = client_id.as_bytes();
		let encode_height = height.encode_vec().map_err(ICS02Error::invalid_encode)?;
		let encode_host_height = host_height.encode_vec().map_err(ICS02Error::invalid_encode)?;

		<ClientUpdateHeight<T>>::insert(encode_client_id, encode_height, encode_host_height);

		Ok(())
	}
}
