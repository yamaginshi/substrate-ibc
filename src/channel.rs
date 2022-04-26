use super::*;
use core::{str::FromStr, time::Duration};

use crate::routing::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd, context::ConnectionReader, error::Error as ICS03Error,
		},
		ics04_channel::{
			channel::ChannelEnd,
			context::{ChannelKeeper, ChannelReader},
			error::Error as Ics04Error,
			packet::{Receipt, Sequence},
		},
		ics05_port::{
			capabilities::{Capability, ChannelCapability, PortCapability},
			context::PortReader,
			error::Error as Ics05Error,
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
		ics26_routing::context::ModuleId,
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ChannelReader for Context<T> {
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, Ics04Error> {
		log::trace!("in channel : [channel_end]");

		let data = <Channels<T>>::get(port_channel_id.0.as_bytes(), port_channel_id.1.encode());

		let channel_end = ChannelEnd::decode_vec(&*data).map_err(|_| {
			Ics04Error::channel_not_found(port_channel_id.clone().0, port_channel_id.clone().1)
		})?;

		log::trace!("in channel : [channel_end] >> channel_end = {:?}", channel_end);
		Ok(channel_end)
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
		log::trace!("in channel : [connection_end]");

		let data = <Connections<T>>::get(connection_id.as_bytes());

		let ret = ConnectionEnd::decode_vec(&*data)
			.map_err(|_| Ics04Error::connection_not_open(connection_id.clone()))?;

		log::trace!("In channel : [connection_end] >> connection_end = {:?}", ret);
		Ok(ret)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
		log::trace!("in channel : [connection_channels]");

		if <ChannelsConnection<T>>::contains_key(conn_id.as_bytes()) {
			let port_and_channel_id = <ChannelsConnection<T>>::get(conn_id.as_bytes());

			let mut result = vec![];

			for item in port_and_channel_id.iter() {
				let port_id = String::from_utf8(item.0.clone())
					.map_err(|e| Ics04Error::invalid_from_utf8(e))?;
				let port_id =
					PortId::from_str(port_id.as_str()).map_err(|e| Ics04Error::identifier(e))?;

				let channel_id = String::from_utf8(item.1.clone())
					.map_err(|e| Ics04Error::invalid_from_utf8(e))?;
				let channel_id = ChannelId::from_str(channel_id.as_str())
					.map_err(|e| Ics04Error::identifier(e))?;

				result.push((port_id, channel_id));
			}

			log::trace!(
				"in channel : [connection_channels] >> Vector<(PortId, ChannelId)> =  {:?}",
				result
			);
			return Ok(result)
		} else {
			return Err(Ics04Error::connection_not_open(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics04Error> {
		log::trace!("in channel : [client_state]");

		let data = <ClientStates<T>>::get(client_id.as_bytes());

		let any_consensus_state = AnyClientState::decode_vec(&*data)
			.map_err(|_| Ics04Error::frozen_client(client_id.clone()))?;

		log::trace!("in channel : [client_state] >> Any client state: {:?}", any_consensus_state);
		Ok(any_consensus_state)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics04Error> {
		log::trace!("in channel : [client_consensus_state]");

		let height = height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?;
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1)
					.map_err(|e| Ics04Error::invalid_decode(e))?;
				log::trace!(
					"in channel: [client_consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(any_consensus_state)
			}
		}
		log::trace!(
			"in channel : [client_consensus_state] >> read about client_id consensus_state error"
		);

		// Err(ICS04Error::frozen_client(client_id.clone()))
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	fn authenticated_capability(&self, port_id: &PortId) -> Result<ChannelCapability, Ics04Error> {
		log::trace!("in channel : [authenticated_capability]");

		match PortReader::lookup_module_by_port(self, port_id) {
			Ok((_, key)) =>
				if !PortReader::authenticate(self, port_id.clone(), &key) {
					Err(Ics04Error::invalid_port_capability())
				} else {
					Ok(Capability::from(key).into())
				},
			Err(e) if e.detail() == Ics05Error::unknown_port(port_id.clone()).detail() =>
				Err(Ics04Error::no_port_capability(port_id.clone())),
			Err(_) => Err(Ics04Error::implementation_specific()),
		}
	}

	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		log::trace!("in channel : [get_next_sequence]");

		let sequence =
			<NextSequenceSend<T>>::get(port_channel_id.0.as_bytes(), port_channel_id.1.encode());

		log::trace!("in channel : [get_next_sequence] >> sequence  = {:?}", sequence);
		Ok(Sequence::from(sequence))
	}

	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		log::trace!("in channel : [get_next_sequence_recv]");

		let sequence =
			<NextSequenceRecv<T>>::get(port_channel_id.0.as_bytes(), port_channel_id.1.encode());

		log::trace!("in channel : [get_next_sequence_recv] >> sequence = {:?}", sequence);
		Ok(Sequence::from(sequence))
	}

	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		log::trace!("in channel : [get_next_sequence_ack]");

		let sequence =
			<NextSequenceAck<T>>::get(port_channel_id.0.as_bytes(), port_channel_id.1.encode());

		log::trace!("in channel : [get_next_sequence_ack] >> sequence = {}", sequence);
		Ok(Sequence::from(sequence))
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<String, Ics04Error> {
		log::trace!("in channel : [get_packet_commitment]");

		let sequence = u64::from(key.2);

		if <PacketCommitment<T>>::contains_key((key.0.as_bytes(), key.1.encode(), sequence)) {
			let data = <PacketCommitment<T>>::get((key.0.as_bytes(), key.1.encode(), sequence));
			let mut data: &[u8] = &data;
			let data =
				Vec::<u8>::decode(&mut data).map_err(|e| Ics04Error::invalid_codec_decode(e))?;
			let result =
				String::from_utf8(data.clone()).map_err(|e| Ics04Error::invalid_from_utf8(e))?;
			log::trace!(
				"in channel : [get_packet_commitment] >> packet_commitment = {:?}, packet_commitment_vec = {:?}",
				result, data
			);
			Ok(result)
		} else {
			log::trace!(
				"in channel : [get_packet_commitment] >> read get packet commitment return None"
			);
			Err(Ics04Error::packet_commitment_not_found(key.2))
		}
	}

	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, Ics04Error> {
		log::trace!("in channel : [get_packet_receipt]");

		let sequence = u64::from(key.2);

		if <PacketReceipt<T>>::contains_key((key.0.as_bytes(), key.1.encode(), sequence.clone())) {
			let data = <PacketReceipt<T>>::get((key.0.as_bytes(), key.1.encode(), sequence));
			let mut data: &[u8] = &data;
			let data =
				Vec::<u8>::decode(&mut data).map_err(|e| Ics04Error::invalid_codec_decode(e))?;
			let data = String::from_utf8(data).map_err(|e| Ics04Error::invalid_from_utf8(e))?;

			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				_ => unreachable!(),
			};
			log::trace!("in channel : [get_packet_receipt] >> packet_receipt = {:?}", data);
			Ok(data)
		} else {
			log::trace!("in channel : [get_packet_receipt] >> read get packet receipt not found");
			Err(Ics04Error::packet_receipt_not_found(key.2))
		}
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<String, Ics04Error> {
		log::trace!("in channel : [get_packet_acknowledgement]");

		let seq = u64::from(key.2);

		if <Acknowledgements<T>>::contains_key((key.0.as_bytes(), key.1.encode(), seq.clone())) {
			let data = <Acknowledgements<T>>::get((key.0.as_bytes(), key.1.encode(), seq));
			let mut data: &[u8] = &data;
			let data =
				Vec::<u8>::decode(&mut data).map_err(|e| Ics04Error::invalid_codec_decode(e))?;
			let result =
				String::from_utf8(data.clone()).map_err(|e| Ics04Error::invalid_from_utf8(e))?;
			log::trace!(
				"in channel : [get_packet_acknowledgement] >> packet_acknowledgement = {:?}",
				result
			);
			Ok(result)
		} else {
			log::trace!(
				"in channel : [get_packet_acknowledgement] >> get acknowledgement not found"
			);
			Err(Ics04Error::packet_acknowledgement_not_found(key.2))
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		log::trace!("in channel: [hash]");

		let r = sp_io::hashing::sha2_256(value.as_bytes());

		let mut tmp = String::new();
		for item in r.iter() {
			tmp.push_str(&format!("{:02x}", item));
		}
		log::trace!("in channel: [hash] >> result = {:?}", tmp.clone());
		tmp
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		log::trace!("in channel: [host_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());

		let current_height: u64 = block_number.parse().unwrap_or_default();

		log::trace!(
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(0, current_height)
		);
		Height::new(0, current_height)
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		log::trace!("in channel: [host_timestamp]");

		// use frame_support::traits::UnixTime;
		// let time = T::TimeProvider::now();
		// let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
		// 	.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		// log::trace!("in channel: [host_timestamp] >> host_timestamp = {:?}",
		// ts.clone().unwrap()); ts.unwrap()
		ClientReader::host_timestamp(self)
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, Ics04Error> {
		log::trace!("in channel: [host_consensus_state]");

		ConnectionReader::host_consensus_state(self, height).map_err(Ics04Error::ics03_connection)
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics04Error> {
		log::trace!("in channel: [pending_host_consensus_stata]");

		ClientReader::pending_host_consensus_state(self)
			.map_err(|e| Ics04Error::ics03_connection(ICS03Error::ics02_client(e)))
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, Ics04Error> {
		log::trace!("in channel: [client_update_time]");

		if <ClientProcessedTimes<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
		) {
			let time = <ClientProcessedTimes<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
			);
			let timestamp =
				String::from_utf8(time).map_err(|e| Ics04Error::invalid_from_utf8(e))?;
			let time: Timestamp = serde_json::from_str(&timestamp)
				.map_err(|e| Ics04Error::invalid_serde_json_decode(e))?;
			Ok(time)
		} else {
			Err(Ics04Error::processed_time_not_found(client_id.clone(), height))
		}
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, Ics04Error> {
		log::trace!("in channel: [client_update_height]");
		if <ClientProcessedHeights<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
		) {
			let host_height = <ClientProcessedHeights<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
			);
			let host_height =
				Height::decode(&mut &host_height[..]).map_err(|e| Ics04Error::invalid_decode(e))?;
			Ok(host_height)
		} else {
			Err(Ics04Error::processed_height_not_found(client_id.clone(), height))
		}
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, Ics04Error> {
		log::trace!("in channel: [channel_counter]");
		Ok(<Pallet<T> as Store>::ChannelCounter::get())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		log::trace!("in channel: [max_expected_time_per_block]");
		Duration::from_secs(6)
	}

	/// Return the module_id along with the capability associated with a given (channel-id, port_id)
	fn lookup_module_by_channel(
		&self,
		channel_id: &ChannelId,
		port_id: &PortId,
	) -> Result<(ModuleId, ChannelCapability), Ics04Error> {
		log::trace!("in channel: [lookup_module_by_channel]");
		// todo
		let module_id = ModuleId::new(format!("ibcmodule").into()).unwrap();
		Ok((module_id, Capability::new().into()))
	}
}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		timestamp: Timestamp,
		heigh: Height,
		data: Vec<u8>,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_packet_commitment]");

		let input = format!("{:?},{:?},{:?}", timestamp, heigh, data);
		let sequence = u64::from(key.2);

		// inser packet commitment key-value
		<PacketCommitment<T>>::insert(
			(key.0.as_bytes().to_vec(), key.1.encode(), sequence),
			ChannelReader::hash(self, input.clone()).encode(),
		);

		// insert packet commitment keys
		let ret = <PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			val.push((key.0.as_bytes().to_vec(), key.1.encode(), sequence));
			Ok(())
		});

		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [delete_packet_commitment]");

		let sequence = u64::from(key.2);

		// delete packet commitment
		<PacketCommitment<T>>::remove((key.0.as_bytes().to_vec(), key.1.encode(), sequence));

		// delete packet commitment keys
		let ret = <PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			let index = val
				.iter()
				.position(|value| value == &(key.0.as_bytes().to_vec(), key.1.encode(), sequence))
				.ok_or(Ics04Error::packet_commitment_keys_not_found())?;
			let ret = val.remove(index);
			assert_eq!(ret, (key.0.as_bytes().to_vec(), key.1.encode(), sequence));
			Ok(())
		});

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_packet_receipt]");

		let receipt = match receipt {
			Receipt::Ok => "Ok".encode(),
		};

		let sequence = u64::from(key.2);

		<PacketReceipt<T>>::insert((key.0.as_bytes().to_vec(), key.1.encode(), sequence), receipt);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_packet_acknowledgement]");

		let ack = format!("{:?}", ack);
		let sequence = u64::from(key.2);

		// store packet acknowledgement key-value
		<Acknowledgements<T>>::insert(
			(key.0.as_bytes().to_vec(), key.1.encode(), sequence),
			ChannelReader::hash(self, ack).encode(),
		);

		// store packet acknowledgement keys
		let ret = <AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			val.push((key.0.as_bytes().to_vec(), key.1.encode(), sequence));
			Ok(())
		});

		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [delete_packet_acknowledgement]");

		let sequence = u64::from(key.2);

		// remove acknowledgements
		<Acknowledgements<T>>::remove((key.0.as_bytes().to_vec(), key.1.encode(), sequence));

		// remove acknowledgement keys for rpc
		let ret = <AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			let index = val
				.iter()
				.position(|value| value == &(key.0.as_bytes().to_vec(), key.1.encode(), sequence))
				.ok_or(Ics04Error::acknowledgements_keys_not_found())?;
			let ret = val.remove(index);
			assert_eq!(&ret, &(key.0.as_bytes().to_vec(), key.1.encode(), sequence));
			Ok(())
		});

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_connection_channels]");

		let conn_id = conn_id.as_bytes().to_vec();

		let port_channel_id = (port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.encode());

		if <ChannelsConnection<T>>::contains_key(conn_id.clone()) {
			log::trace!("in channel: [store_connection_channels] >> insert port_channel_id");
			// if connection_identifier exist
			let ret =
				<ChannelsConnection<T>>::try_mutate(conn_id, |val| -> Result<(), Ics04Error> {
					val.push(port_channel_id);
					Ok(())
				})
				.map_err(|_| Ics04Error::invalid_store_channels_connection());
		} else {
			// if connection_identifier no exist
			log::trace!("in channel: [store_connection_channels] >> init ChannelsConnection");
			let temp_connection_channels = vec![port_channel_id];
			<ChannelsConnection<T>>::insert(conn_id, temp_connection_channels);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_channel]");

		let channel_end = channel_end.encode_vec().map_err(|_| Ics04Error::invalid_encode())?;

		// store channels key-value
		<Channels<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.encode(),
			channel_end,
		);

		// store channels keys for rpc
		let ret = <ChannelsKeys<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			if let Some(_value) = val.iter().find(|&x| {
				x == &(port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.encode())
			}) {
			} else {
				val.push((port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.encode()));
			}

			Ok(())
		})
		.map_err(|_| Ics04Error::invalid_store_channels_keys());

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_next_sequence_send]");

		let sequence = u64::from(seq);

		<NextSequenceSend<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.encode(),
			sequence,
		);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_next_sequence_recv]");

		let sequence = u64::from(seq);

		<NextSequenceRecv<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.encode(),
			sequence,
		);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		log::trace!("in channel: [store_next_sequence_ack]");

		let sequence = u64::from(seq);

		<NextSequenceAck<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.encode(),
			sequence,
		);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::trace!("in channel: [increase_channel_counter]");

		let ret = <ChannelCounter<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			let new = val.checked_add(1).ok_or(Ics04Error::ivalid_increase_channel_counter())?;
			*val = new;
			Ok(())
		});
	}
}
