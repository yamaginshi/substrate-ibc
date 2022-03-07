#![cfg_attr(not(feature = "std"), no_std)]
// TODO to remove
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

//! # IBC Module
//!
//! This module implements the standard [IBC protocol](https://github.com/cosmos/ics).
//!
//! ## Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! This project is currently in an early stage and will eventually be submitted to upstream.
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs), which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).
//!
//! The chain specific logic of the modules in ICS spec implemented::
//! * ics-002-client-semantics
//! * ics-003-connection-semantics
//! * ics-004-channel-and-packet-semantics
//! * ics-005-port-allocation
//! * ics-010-grandpa-client
//! * ics-018-relayer-algorithms
//! * ics-025-handler-interface
//! * ics-026-routing-module
//!
//! ### Terminology
//!
//! Please refer to [IBC Terminology](https://github.com/cosmos/ibc/blob/a983dd86815175969099d041906f6a14643e51ef/ibc/1_IBC_TERMINOLOGY.md).
//!
//! ### Goals
//!
//! This IBC module handles authentication, transport, and ordering of structured data packets
//! relayed between modules on separate machines.
//!
//! ## Interface
//!
//! ###  Public Functions
//!
//! * `deliver` - `ibc::ics26_routing::handler::deliver` Receives datagram transmitted from relayers/users, and pass to ICS26 router to look for the correct handler.
//!
//! ## Usage
//! Please refer to section "How to Interact with the Pallet" in the repository's README.md

extern crate alloc;

pub use pallet::*;

use alloc::{format, string::String};
use crate::alloc::string::ToString;
use beefy_light_client::commitment;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_system::ensure_signed;
use ibc::core::ics02_client::client_state::AnyClientState;
use ibc::core::ics02_client::client_consensus::AnyConsensusState;
use ibc::core::ics02_client::height;
use ibc::clients::ics10_grandpa::client_state::ClientState;
use ibc::clients::ics10_grandpa::help;
use ibc::clients::ics10_grandpa::help::{BlockHeader, Commitment};
use ibc::core::ics24_host::identifier::ChainId as ICS24ChainId;
pub use routing::ModuleCallbacks;
use scale_info::{prelude::vec, TypeInfo};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use tendermint_proto::Protobuf;

mod channel;
mod client;
mod connection;
pub mod event;
mod port;
mod routing;

/// A struct corresponds to `Any` in crate "prost-types", used in ibc-rs.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<prost_types::Any> for Any {
	fn from(any: prost_types::Any) -> Self {
		Self { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::event::primitive::Sequence;
	use event::primitive::{
		ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height,
		Packet, PortId, Timestamp,
	};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::events::IbcEvent;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ModuleCallbacks: routing::ModuleCallbacks;
		type TimeProvider: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::type_value]
	pub fn default_client_state_keys() -> Vec<Vec<u8>> {
		vec![]
	}

	#[pallet::storage]
	/// client_id vector
	pub type ClientStatesKeys<T: Config> =
		StorageValue<_, Vec<Vec<u8>>, ValueQuery, default_client_state_keys>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientProcessedHeights<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// client_id => Vector<(Height, ConsensusState)>
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// connection_id => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::type_value]
	pub fn default_connection_keys() -> Vec<Vec<u8>> {
		vec![]
	}

	#[pallet::storage]
	/// connection_id vector
	pub type ConnectionsKeys<T: Config> =
		StorageValue<_, Vec<Vec<u8>>, ValueQuery, default_connection_keys>;

	#[pallet::storage]
	/// (port_identifier, channel_identifier) => ChannelEnd
	pub type Channels<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::type_value]
	pub fn default_channels_keys() -> Vec<(Vec<u8>, Vec<u8>)> {
		vec![]
	}

	#[pallet::storage]
	/// vector of (port_id, channel_id)
	pub type ChannelsKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery, default_channels_keys>;

	#[pallet::storage]
	/// connection_id => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::type_value]
	pub fn default_acknowledgements_keys() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> {
		vec![]
	}

	#[pallet::storage]
	/// vector of (port_identifier, channel_identifier, sequence)
	pub type AcknowledgementsKeys<T: Config> = StorageValue<
		_,
		Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
		ValueQuery,
		default_acknowledgements_keys,
	>;

	#[pallet::storage]
	/// client_id => client_type
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::type_value]
	pub fn default_client_counter() -> u64 {
		0u64
	}

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery, default_client_counter>;

	#[pallet::type_value]
	pub fn default_connection_counter() -> u64 {
		0u64
	}

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	/// connection counter
	pub type ConnectionCounter<T: Config> =
		StorageValue<_, u64, ValueQuery, default_connection_counter>;

	#[pallet::type_value]
	pub fn default_channel_counter() -> u64 {
		0u64
	}

	#[pallet::storage]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery, default_channel_counter>;

	#[pallet::storage]
	/// client_id => connection_id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of (timestamp, heigh, packet)
	pub type PacketCommitment<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::type_value]
	pub fn default_packet_commitment_keys() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> {
		vec![]
	}

	#[pallet::storage]
	/// vector of (port_id, channel_id, sequence)
	pub type PacketCommitmentKeys<T: Config> = StorageValue<
		_,
		Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
		ValueQuery,
		default_packet_commitment_keys,
	>;

	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => sendpacket event
	pub type SendPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => writeack event
	pub type WriteAckPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::type_value]
	pub fn defaultlatest_height() -> Vec<u8> {
		let height = ibc::Height::default();

		height.encode_vec().unwrap()
	}

	#[pallet::storage]
	/// store latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery, defaultlatest_height>;

	#[pallet::type_value]
	pub fn default_old_height() -> u64 {
		0
	}

	#[pallet::storage]
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery, default_old_height>;

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewBlock(Height),

		CreateClient(Height, ClientId, ClientType, Height),
		UpdateClient(Height, ClientId, ClientType, Height),
		UpdateClientState(Height, EventClientState),
		UpgradeClient(Height, ClientId, ClientType, Height),
		ClientMisbehaviour(Height, ClientId, ClientType, Height),
		OpenInitConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenTryConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenAckConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenConfirmConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
		OpenInitChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenTryChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenAckChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		CloseInitChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		CloseConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		SendPacket(Height, Packet),
		ReceivePacket(Height, Packet),
		WriteAcknowledgement(Height, Packet, Vec<u8>),
		AcknowledgePacket(Height, Packet),
		TimeoutPacket(Height, Packet),
		TimeoutOnClosePacket(Height, Packet),
		Empty(Vec<u8>),
		ChainError(Vec<u8>),
	}

	/// Convert events of ibc-rs to the corresponding events in substrate-ibc
	impl<T: Config> From<ibc::events::IbcEvent> for Event<T> {
		fn from(value: ibc::events::IbcEvent) -> Self {
			match value {
				ibc::events::IbcEvent::NewBlock(value) => Event::NewBlock(value.height.into()),
				ibc::events::IbcEvent::CreateClient(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::CreateClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::UpdateClient(value) => {
					let height = value.common.height;
					let client_id = value.common.client_id;
					let client_type = value.common.client_type;
					let consensus_height = value.common.consensus_height;
					Event::UpdateClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				// TODO! Upgrade client events are not currently being used
				// UpgradeClient(
				// 	height: Height,
				// 	client_id: ClientId,
				// 	client_type: ClientType,
				// 	consensus_height: Height,
				// )
				ibc::events::IbcEvent::UpgradeClient(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::UpgradeClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::ClientMisbehaviour(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::ClientMisbehaviour(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::OpenInitConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenInitConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenTryConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenTryConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenAckConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenAckConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenConfirmConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenConfirmConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenInitChannel(value) => {
					let height = value.attributes().height;
					let port_id = value.attributes().port_id.clone();
					let channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					let connection_id = value.attributes().connection_id.clone();
					let counterparty_port_id = value.attributes().counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					Event::OpenInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenTryChannel(value) => {
					let height = value.attributes().height;
					let port_id = value.attributes().port_id.clone();
					let channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					let connection_id = value.attributes().connection_id.clone();
					let counterparty_port_id = value.attributes().counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					Event::OpenTryChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenAckChannel(value) => {
					let height = value.attributes().height;
					let port_id = value.attributes().port_id.clone();
					let channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					let connection_id = value.attributes().connection_id.clone();
					let counterparty_port_id = value.attributes().counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.attributes().channel_id.clone().map(|val| val.into());
					Event::OpenAckChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenConfirmChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::OpenConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::CloseInitChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::CloseInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::CloseConfirmChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::CloseConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::SendPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::SendPacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::ReceivePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::ReceivePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::WriteAcknowledgement(value) => {
					let height = value.height;
					let packet = value.packet;
					let ack = value.ack;
					Event::WriteAcknowledgement(height.into(), packet.into(), ack)
				},
				ibc::events::IbcEvent::AcknowledgePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::AcknowledgePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::TimeoutPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutPacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::TimeoutOnClosePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutOnClosePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::Empty(value) => Event::Empty(value.as_bytes().to_vec()),
				ibc::events::IbcEvent::ChainError(value) => {
					Event::ChainError(value.as_bytes().to_vec())
				},
				_ => unimplemented!(),
			}
		}
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		///receive mmr root block number less than client_state.latest_commitment.block_number !
		NotLaststCommitment,
		///verify mmr root failure
		///update the beefy light client failure!
		VerifyMmrRootFailure,
	}

	// mock client state
	fn mock_client_state() -> ClientState {
		// mock light client
		let public_keys = vec![
			String::from("0x020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1"), // Alice
		];
		let lc = beefy_light_client::new(public_keys);
		log::info!("mock beefy light client: {:?}", lc);

		//mock client state
		let epoch_number = 10;
		// let chain_id = ICS24ChainId::new(String::from("chainA"), epoch_number);
		let chain_id = ICS24ChainId::new(String::from("chainA"), epoch_number);
		let client_state = ClientState {
			chain_id: chain_id.clone(),
			block_number: u32::default(),
			frozen_height: None,
			block_header: BlockHeader::default(),
			latest_commitment: Commitment::default(),
			validator_set: lc.validator_set.clone().into(),
		};
		log::info!("mock client_state : {:?}", client_state);

		client_state
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for all of the IBC request(except MMR root update).
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		///
		/// Example of invoking this function via subxt
		///
		/// ```ignore
		///     let api = client.to_runtime_api::<ibc_node::RuntimeApi<ibc_node::DefaultConfig>>();
		///
		///     let result = api
		///         .tx()
		///         .ibc()
		///         .deliver(msg, 0)
		///         .sign_and_submit(&signer)
		///         .await?;
		/// ```
		#[pallet::weight(0)]
		pub fn deliver(origin: OriginFor<T>, messages: Vec<Any>, tmp: u8) -> DispatchResult {
			log::info!("in deliver");
			for item in messages.iter() {
				log::info!("Message type: {:?}", String::from_utf8(item.type_url.clone()).unwrap());
			}

			let _sender = ensure_signed(origin)?;
			let mut ctx = routing::Context { _pd: PhantomData::<T>, tmp };
			let messages = messages
				.iter()
				.map(|message| prost_types::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value.clone(),
				})
				.collect();
			let result = ibc::core::ics26_routing::handler::deliver(&mut ctx, messages).unwrap_or_else(|value| vec![IbcEvent::ChainError(value.to_string())]);

			log::info!("result: {:?}", result);

			for event in result {
				log::info!("Event: {:?}", event);
				Self::deposit_event(event.clone().into());
				Self::store_latest_height(event.clone());
			}

			Ok(())
		}

		/// Update the MMR root stored in client_state
		/// Example of invoking this function via subxt
		///
		/// ```ignore
		///     let api = client.to_runtime_api::<ibc_node::RuntimeApi<ibc_node::DefaultConfig>>();
		///
		///     let result = api
		///         .tx()
		///         .ibc()
		///         .update_client_state(encode_client_id, encode_mmr_root)
		///         .sign_and_submit(&signer)
		///         .await?;
		/// ```
		#[pallet::weight(0)]
		pub fn update_client_state(
			origin: OriginFor<T>,
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResult {
			log::info!("received update_client_state request.");
			let _who = ensure_signed(origin)?;

			// check if the client id exist?
			let client_id_str = String::from_utf8(client_id.clone()).unwrap();
			log::info!("received client id is {:?}", client_id_str);

			let decode_received_mmr_root = help::MmrRoot::decode(&mut &mmr_root[..]).unwrap();
			log::info!("receive decode mmr root is {:?}", decode_received_mmr_root);

			let mut client_state = ClientState::default();

			if !<ClientStates<T>>::contains_key(client_id.clone()) {
				log::info!("in update_client_state: {:?} client_state not found !", client_id_str);
				return core::result::Result::Err(DispatchError::Other("client id not found"));
			} else {
				// get client state from chain storage
				let data = <ClientStates<T>>::get(client_id.clone());
				let any_client_state = AnyClientState::decode_vec(&*data).unwrap();
				client_state = match any_client_state {
					AnyClientState::Grandpa(value) => value,
					_ => unimplemented!(),
				};

				log::info!(
					"in update_client_state : get client_state from chain storage: {:?}",
					client_state
				);
			}

			let signed_commitment =
				commitment::SignedCommitment::from(decode_received_mmr_root.signed_commitment);
			let rev_block_number = signed_commitment.clone().commitment.block_number;
			if rev_block_number <= client_state.latest_commitment.block_number {
				log::info!("receive mmr root block number({}) less than client_state.latest_commitment.block_number({})",
				rev_block_number,client_state.latest_commitment.block_number);

				return core::result::Result::Err(DispatchError::Other("receive mmr root block number less than client_state.latest_commitment.block_number !"));
			}
			// build new beefy light client by client_state
			let mut light_client = beefy_light_client::LightClient {
				latest_commitment: Some(client_state.latest_commitment.clone().into()),
				validator_set: client_state.validator_set.clone().into(),
				in_process_state: None,
			};
			log::info!(
				"build new beefy_light_client from client_state store in chain \n {:?}",
				light_client
			);

			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs = decode_received_mmr_root.validator_merkle_proofs;
			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs: Vec<beefy_light_client::ValidatorMerkleProof> = validator_proofs
				.into_iter()
				.map(|validator_proof| validator_proof.into())
				.collect();

			// encode signed_commitment
			let encoded_signed_commitment =
				commitment::SignedCommitment::encode(&signed_commitment);

			let mmr_leaf = decode_received_mmr_root.mmr_leaf;
			let mmr_leaf_proof = decode_received_mmr_root.mmr_leaf_proof;

			// verfiy mmr proof and update lc state
			let result = light_client.update_state(
				&encoded_signed_commitment,
				&validator_proofs,
				&mmr_leaf,
				&mmr_leaf_proof,
			);

			// TODO
			// we can update mmr root save into consensus_state
			match result {
				Ok(_) => {
					log::info!("update the beefy light client sucesse! and the beefy light client state is : {:?} \n",light_client);

					// update client_client block number and latest commitment
					let latest_commitment = light_client.latest_commitment.unwrap();
					client_state.block_number = latest_commitment.block_number;
					client_state.latest_commitment =
						help::Commitment::from(latest_commitment.clone());

					// update validator_set
					client_state.validator_set =
						help::ValidatorSet::from(light_client.validator_set.clone());

					// update block header
					client_state.block_header = decode_received_mmr_root.block_header.clone();

					// save to chain
					let any_client_state = AnyClientState::Grandpa(client_state.clone());
					let data = any_client_state.encode_vec().unwrap();
					// store client states key-value
					<ClientStates<T>>::insert(client_id.clone(), data);

					// store client states keys
					<ClientStatesKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
						if let Some(_value) = val.iter().find(|&x| x == &client_id.clone()) {
						} else {
							val.push(client_id.clone());
						}
						Ok(())
					})
					.expect("store client_state keys error");

					// save every time mmr root into consensus state 
					let grandpa_consensus_state = ibc::clients::ics10_grandpa::consensus_state::ConsensusState::new(client_state.latest_commitment.payload.clone());
					
					let consensus_stata_data = AnyConsensusState::Grandpa(grandpa_consensus_state).encode_vec().unwrap();
					
					let consensus_state_height = ibc::Height::new(0, latest_commitment.block_number as u64).encode_vec().unwrap();
					if <ConsensusStates<T>>::contains_key(client_id.clone()) {
						// if consensus_state is no empty use push insert an exist ConsensusStates
						<ConsensusStates<T>>::try_mutate(
							client_id.clone(),
							|val| -> Result<(), &'static str> {
								val.push((consensus_state_height, consensus_stata_data));
								Ok(())
							},
						)
							.expect("store consensus state error");
					} else {
						// if consensus state is empty insert a new item.
						<ConsensusStates<T>>::insert(client_id.clone(), vec![(consensus_state_height, consensus_stata_data)]);
					}

					// emit update state sucesse event
					let event_height = Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};
					let event_client_state = EventClientState::from(client_state);
					Self::deposit_event(Event::<T>::UpdateClientState(
						event_height,
						event_client_state,
					));
				},
				Err(e) => {
					log::info!("update the beefy light client failure! : {:?}", e);

					return core::result::Result::Err(DispatchError::Other(
						"update the beefy light client failure!",
					));
				},
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// update the latest height of a client
		fn store_latest_height(ibc_event: IbcEvent) {
			match ibc_event {
				IbcEvent::Empty(_value) => {
					log::info!("ibc event: {}", "Empty");
				},
				IbcEvent::NewBlock(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::SendPacket(value) => {
					// store height
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);

					// store send-packet
					let _value = value.clone();
					let packet = Packet {
						sequence: Sequence::from(_value.packet.sequence),
						source_channel: ChannelId::from(_value.packet.source_channel),
						source_port: PortId::from(_value.packet.source_port),
						destination_channel: ChannelId::from(_value.packet.destination_channel),
						destination_port: PortId::from(_value.packet.destination_port),
						data: _value.packet.data,
						timeout_timestamp: Timestamp::from(_value.packet.timeout_timestamp),
						timeout_height: Height::from(_value.packet.timeout_height),
					};
					let packet = packet.to_ibc_packet().encode_vec().unwrap();

					let port_id = value.packet.source_port.as_bytes().to_vec();
					let channel_id = value.packet.source_channel.as_bytes().to_vec();

					<SendPacketEvent<T>>::insert(
						(port_id, channel_id, u64::from(value.packet.sequence)),
						packet,
					);
				},
				IbcEvent::WriteAcknowledgement(value) => {
					// store height
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);

					// store ack
					let port_id = value.packet.source_port.as_bytes().to_vec();
					let channel_id = value.packet.source_channel.as_bytes().to_vec();
					let sequence = u64::from(value.packet.sequence);
					let ack = value.ack;
					// store.Set((portID, channelID, sequence), ackHash)
					<WriteAckPacketEvent<T>>::insert((port_id, channel_id, sequence), ack)
				},
				IbcEvent::UpdateClient(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ReceivePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseConfirmChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CreateClient(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::UpgradeClient(value) => {
					let height = value.0.height.clone().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ClientMisbehaviour(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseInitChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::AcknowledgePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutPacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutOnClosePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ChainError(_value) => {
					log::info!("Ibc event: {}", "chainError");
				},
			}
		}
	}
}
