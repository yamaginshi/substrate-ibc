use super::*;
use crate::utils::apply_prefix_and_encode;
use ibc::{
	core::{
		ics04_channel::packet::Sequence,
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath,
				ClientTypePath, CommitmentsPath, ConnectionsPath, ReceiptsPath, SeqAcksPath,
				SeqRecvsPath, SeqSendsPath,
			},
		},
	},
	Height as ICSHeight,
};
use scale_info::prelude::{format, string::String};
use sp_core::H256;

impl<T: Config> Pallet<T> {
	pub fn build_trie_inputs() -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error<T>> {
		let mut inputs: Vec<(Vec<u8>, Vec<u8>)> = vec![];

		Self::build_client_states_trie(&mut inputs)?;
		Self::build_consensus_states_trie(&mut inputs)?;
		Self::build_connection_ends_trie(&mut inputs)?;
		Self::build_channel_ends_trie(&mut inputs)?;
		Self::build_sequences_trie(&mut inputs)?;
		Self::build_packet_commitment_trie(&mut inputs)?;
		Self::build_packet_acknowledgements_trie(&mut inputs)?;
		Self::build_packet_receipts_trie(&mut inputs)?;

		Ok(inputs)
	}

	pub fn build_ibc_state_root() -> Result<H256, Error<T>> {
		let inputs = Self::build_trie_inputs()?;
		Ok(sp_io::trie::blake2_256_root(inputs, sp_core::storage::StateVersion::V0))
	}

	pub fn extract_ibc_state_root() -> Result<Vec<u8>, Error<T>> {
		let root = Self::build_ibc_state_root()?;
		Ok(root.as_bytes().to_vec())
	}

	/// insert client type and client state in trie
	fn build_client_states_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for (client_id, client_state) in <ClientStates<T>>::iter() {
			let client_type = <Clients<T>>::get(&client_id);

			let client_id = ClientId::from_str(
				String::from_utf8(client_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let client_state_path = format!("{}", ClientStatePath(client_id.clone()));
			let client_type_path = format!("{}", ClientTypePath(client_id.clone()));

			let client_state_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![client_state_path]);

			let client_type_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![client_type_path]);

			// insert client state into trie
			inputs.push((client_state_key, client_state));
			// insert client type into trie
			inputs.push((client_type_key, client_type));
		}

		Ok(())
	}

	/// insert consensus states into trie
	fn build_consensus_states_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for (client_id, height, consensus_state) in <ConsensusStates<T>>::iter() {
			let client_id = ClientId::from_str(
				String::from_utf8(client_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let height = ICSHeight::decode(&mut &*height).map_err(|_| Error::<T>::InvalidDecode)?;

			let client_consensus_state_path = ClientConsensusStatePath {
				client_id,
				epoch: height.revision_number(),
				height: height.revision_height(),
			};
			let client_consensus_state_path = format!("{}", client_consensus_state_path);
			let client_consensus_state_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![client_consensus_state_path]);

			// insert consensus_state into trie
			inputs.push((client_consensus_state_key, consensus_state));
		}

		Ok(())
	}

	/// insert connection ends into trie
	fn build_connection_ends_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for (connection_id, connection_end) in <Connections<T>>::iter() {
			let connection_id = ConnectionId::from_str(
				String::from_utf8(connection_id)
					.as_ref()
					.map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let connections_path = format!("{}", ConnectionsPath(connection_id));

			let connection_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![connections_path]);

			// insert connection end into trie
			inputs.push((connection_key, connection_end));
		}
		Ok(())
	}

	/// insert channel ends into trie
	fn build_channel_ends_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for (port_id, channel_id, channel_end) in <Channels<T>>::iter() {
			let port_id = PortId::from_str(
				String::from_utf8(port_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let channel_id = ChannelId::from_str(
				String::from_utf8(channel_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let channel_end_path = format!("{}", ChannelEndsPath(port_id, channel_id));

			let channel_end_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![channel_end_path]);

			// insert channel end into trie
			inputs.push((channel_end_key, channel_end));
		}

		Ok(())
	}

	/// insert sequences in trie
	fn build_sequences_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for (port_id, channel_id, _) in <Channels<T>>::iter() {
			let next_sequence_send = <NextSequenceSend<T>>::get(&port_id, &channel_id);
			let next_sequence_recv = <NextSequenceRecv<T>>::get(&port_id, &channel_id);
			let next_sequence_ack = <NextSequenceAck<T>>::get(&port_id, &channel_id);

			let port_id = PortId::from_str(
				String::from_utf8(port_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let channel_id = ChannelId::from_str(
				String::from_utf8(channel_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;

			let next_sequence_send_path =
				format!("{}", SeqSendsPath(port_id.clone(), channel_id.clone()));
			let next_sequence_recv_path =
				format!("{}", SeqRecvsPath(port_id.clone(), channel_id.clone()));
			let next_sequence_ack_path =
				format!("{}", SeqAcksPath(port_id.clone(), channel_id.clone()));

			let next_sequence_send_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![next_sequence_send_path]);
			let next_sequence_recv_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![next_sequence_recv_path]);
			let next_sequence_ack_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![next_sequence_ack_path]);

			// insert next sequence send into trie
			inputs.push((next_sequence_send_key, next_sequence_send.encode()));
			// insert next sequence recv into trie
			inputs.push((next_sequence_recv_key, next_sequence_recv.encode()));
			// insert next sequence ack into trie
			inputs.push((next_sequence_ack_key, next_sequence_ack.encode()));
		}
		Ok(())
	}

	/// insert packet commitment trie
	fn build_packet_commitment_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for ((port_id, channel_id, sequence), packet_commitment) in <PacketCommitments<T>>::iter() {
			let channel_id = ChannelId::from_str(
				String::from_utf8(channel_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let port_id = PortId::from_str(
				String::from_utf8(port_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let sequence = Sequence::from(sequence);

			let packet_commitment_path =
				format!("{}", CommitmentsPath { port_id, channel_id, sequence });

			let packet_commitment_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![packet_commitment_path]);

			inputs.push((packet_commitment_key, packet_commitment));
		}

		Ok(())
	}

	/// insert packet acknowledgement into trie
	fn build_packet_acknowledgements_trie(
		inputs: &mut Vec<(Vec<u8>, Vec<u8>)>,
	) -> Result<(), Error<T>> {
		for ((port_id, channel_id, sequence), acknowledgement) in <Acknowledgements<T>>::iter() {
			let channel_id = ChannelId::from_str(
				String::from_utf8(channel_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let port_id = PortId::from_str(
				String::from_utf8(port_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let sequence = Sequence::from(sequence);

			let acknowledgement_path = format!("{}", AcksPath { port_id, channel_id, sequence });

			let acknowledgement_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![acknowledgement_path]);

			inputs.push((acknowledgement_key, acknowledgement));
		}
		Ok(())
	}

	/// insert packet receipt into trie
	fn build_packet_receipts_trie(inputs: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error<T>> {
		for ((port_id, channel_id, sequence), packet_receipt) in <PacketReceipts<T>>::iter() {
			let channel_id = ChannelId::from_str(
				String::from_utf8(channel_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let port_id = PortId::from_str(
				String::from_utf8(port_id).as_ref().map_err(|_| Error::<T>::InvalidDecode)?,
			)
			.map_err(|_| Error::<T>::InvalidDecode)?;
			let sequence = Sequence::from(sequence);

			let packet_receipt_path = format!("{}", ReceiptsPath { port_id, channel_id, sequence });

			let packet_receipt_key =
				apply_prefix_and_encode(T::CONNECTION_PREFIX, vec![packet_receipt_path]);

			inputs.push((packet_receipt_key, packet_receipt));
		}

		Ok(())
	}
}
