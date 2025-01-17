use core::marker::PhantomData;

use crate::{Config, Error, Event};
pub use alloc::{
	format,
	string::{String, ToString},
};
use codec::{Decode, Encode};
use ibc::{core::ics26_routing, events::IbcEvent as RawIbcEvent};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::{str::FromStr, vec::Vec};

impl<T: Config> TryFrom<RawIbcEvent> for Event<T> {
	type Error = Error<T>;

	fn try_from(raw_ibc_event: RawIbcEvent) -> Result<Self, Self::Error> {
		match raw_ibc_event {
			RawIbcEvent::CreateClient(create_client) => {
				let client_id = create_client.client_id().clone();
				let client_type = create_client.client_type().clone();
				let consensus_height = create_client.consensus_height().clone();

				Ok(Event::<T>::CreateClient { client_id, client_type, consensus_height })
			},
			RawIbcEvent::UpdateClient(update_client) => {
				let client_id = update_client.client_id().clone();
				let client_type = update_client.client_type().clone();
				let consensus_height = update_client.consensus_height().clone();
				let consensus_heights = update_client.consensus_heights().to_vec();
				let header = update_client.header();

				Ok(Event::<T>::UpdateClient {
					client_id,
					client_type,
					consensus_height,
					consensus_heights,
					header: header.clone().into(),
				})
			},
			// Upgrade client events are not currently being used
			RawIbcEvent::UpgradeClient(upgrade_client) => {
				let client_id = upgrade_client.client_id().clone();
				let client_type = upgrade_client.client_type().clone();
				let consensus_height = upgrade_client.consensus_height().clone();

				Ok(Event::<T>::UpgradeClient { client_id, client_type, consensus_height })
			},
			RawIbcEvent::ClientMisbehaviour(client_misbehaviour) => {
				let client_id = client_misbehaviour.client_id().clone();
				let client_type = client_misbehaviour.client_type().clone();

				Ok(Event::<T>::ClientMisbehaviour { client_id, client_type })
			},
			RawIbcEvent::OpenInitConnection(open_init_connection) => {
				let connection_id = open_init_connection.connection_id().clone();
				let client_id = open_init_connection.client_id().clone();
				let counterparty_connection_id =
					open_init_connection.counterparty_connection_id().map(|value| value.clone());
				let counterparty_client_id = open_init_connection.counterparty_client_id().clone();

				Ok(Event::<T>::OpenInitConnection {
					connection_id,
					client_id,
					counterparty_connection_id,
					counterparty_client_id,
				})
			},
			RawIbcEvent::OpenTryConnection(open_try_connection) => {
				let connection_id = open_try_connection.connection_id().clone();
				let client_id = open_try_connection.client_id().clone();
				let counterparty_connection_id =
					open_try_connection.counterparty_connection_id().map(|value| value.clone());
				let counterparty_client_id = open_try_connection.counterparty_client_id().clone();

				Ok(Event::<T>::OpenTryConnection {
					connection_id,
					client_id,
					counterparty_connection_id,
					counterparty_client_id,
				})
			},
			RawIbcEvent::OpenAckConnection(open_ack_connection) => {
				let connection_id = open_ack_connection.connection_id().clone();
				let client_id = open_ack_connection.client_id().clone();
				let counterparty_connection_id =
					open_ack_connection.counterparty_connection_id().map(|value| value.clone());
				let counterparty_client_id = open_ack_connection.counterparty_client_id().clone();

				Ok(Event::<T>::OpenAckConnection {
					connection_id,
					client_id,
					counterparty_connection_id,
					counterparty_client_id,
				})
			},
			RawIbcEvent::OpenConfirmConnection(open_confirm_connection) => {
				let connection_id = open_confirm_connection.connection_id().clone();
				let client_id = open_confirm_connection.client_id().clone();
				let counterparty_connection_id =
					open_confirm_connection.counterparty_connection_id().map(|value| value.clone());
				let counterparty_client_id =
					open_confirm_connection.counterparty_client_id().clone();

				Ok(Event::<T>::OpenConfirmConnection {
					connection_id,
					client_id,
					counterparty_connection_id,
					counterparty_client_id,
				})
			},
			RawIbcEvent::OpenInitChannel(open_init_channel) => {
				let port_id = open_init_channel.port_id().clone();
				let channel_id = open_init_channel.channel_id().clone();
				let counterparty_port_id = open_init_channel.counterparty_port_id().clone();
				let connection_id = open_init_channel.connection_id().clone();
				let version = open_init_channel.version().clone();

				Ok(Event::<T>::OpenInitChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					connection_id,
					version,
				})
			},
			RawIbcEvent::OpenTryChannel(open_try_channel) => {
				let port_id = open_try_channel.port_id().clone();
				let channel_id = open_try_channel.channel_id().clone();
				let counterparty_port_id = open_try_channel.counterparty_port_id().clone();
				let counterparty_channel_id = open_try_channel.counterparty_channel_id().clone();
				let connection_id = open_try_channel.connection_id().clone();
				let version = open_try_channel.version().clone();

				Ok(Event::<T>::OpenTryChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					connection_id,
					version,
				})
			},
			RawIbcEvent::OpenAckChannel(open_ack_channel) => {
				let port_id = open_ack_channel.port_id().clone();
				let channel_id = open_ack_channel.channel_id().clone();
				let counterparty_port_id = open_ack_channel.counterparty_port_id().clone();
				let counterparty_channel_id = open_ack_channel.counterparty_channel_id().clone();
				let connection_id = open_ack_channel.connection_id().clone();

				Ok(Event::<T>::OpenAckChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					connection_id,
				})
			},
			RawIbcEvent::OpenConfirmChannel(open_confirm_channel) => {
				let port_id = open_confirm_channel.port_id().clone();
				let channel_id = open_confirm_channel.channel_id().clone();
				let counterparty_port_id = open_confirm_channel.counterparty_port_id().clone();
				let counterparty_channel_id =
					open_confirm_channel.counterparty_channel_id().clone();
				let connection_id = open_confirm_channel.connection_id().clone();

				Ok(Event::<T>::OpenConfirmChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					connection_id,
				})
			},
			RawIbcEvent::CloseInitChannel(close_init_channel) => {
				let port_id = close_init_channel.port_id().clone();
				let channel_id = close_init_channel.channel_id().clone();
				let counterparty_port_id = close_init_channel.counterparty_port_id().clone();
				let counterparty_channel_id = close_init_channel.counterparty_channel_id().clone();
				let connection_id = close_init_channel.connection_id().clone();

				Ok(Event::<T>::CloseInitChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					connection_id,
				})
			},
			RawIbcEvent::CloseConfirmChannel(close_confirm_channel) => {
				let port_id = close_confirm_channel.port_id().clone();
				let channel_id = close_confirm_channel.channel_id().clone();
				let counterparty_port_id = close_confirm_channel.counterparty_port_id().clone();
				let counterparty_channel_id =
					close_confirm_channel.counterparty_channel_id().clone();
				let connection_id = close_confirm_channel.connection_id().clone();

				Ok(Event::<T>::CloseConfirmChannel {
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					connection_id,
				})
			},
			RawIbcEvent::SendPacket(send_packet) => {
				let packet_data = send_packet.packet_data().to_vec();
				let timeout_height = send_packet.timeout_height().clone();
				let timeout_timestamp = send_packet.timeout_timestamp().clone();
				let sequence = send_packet.sequence().clone();
				let src_port_id = send_packet.src_port_id().clone();
				let src_channel_id = send_packet.src_channel_id().clone();
				let dst_port_id = send_packet.dst_port_id().clone();
				let dst_channel_id = send_packet.dst_channel_id().clone();
				let channel_ordering = send_packet.channel_ordering().clone();
				let src_connection_id = send_packet.src_connection_id().clone();

				Ok(Event::<T>::SendPacket {
					packet_data,
					timeout_height,
					timeout_timestamp,
					sequence,
					src_port_id,
					src_channel_id,
					dst_port_id,
					dst_channel_id,
					channel_ordering,
					src_connection_id,
				})
			},
			RawIbcEvent::ReceivePacket(receiver_packet) => {
				let packet_data = receiver_packet.packet_data().to_vec();
				let timeout_height = receiver_packet.timeout_height().clone();
				let timeout_timestamp = receiver_packet.timeout_timestamp().clone();
				let sequence = receiver_packet.sequence().clone();
				let src_port_id = receiver_packet.src_port_id().clone();
				let src_channel_id = receiver_packet.src_channel_id().clone();
				let dst_port_id = receiver_packet.dst_port_id().clone();
				let dst_channel_id = receiver_packet.dst_channel_id().clone();
				let channel_ordering = receiver_packet.channel_ordering().clone();
				let dst_connection_id = receiver_packet.dst_connection_id().clone();

				Ok(Event::<T>::ReceivePacket {
					packet_data,
					timeout_height,
					timeout_timestamp,
					sequence,
					src_port_id,
					src_channel_id,
					dst_port_id,
					dst_channel_id,
					channel_ordering,
					dst_connection_id,
				})
			},
			RawIbcEvent::WriteAcknowledgement(write_acknowledgement) => {
				let packet_data = write_acknowledgement.packet_data().to_vec();
				let timeout_height = write_acknowledgement.timeout_height().clone();
				let timeout_timestamp = write_acknowledgement.timeout_timestamp().clone();
				let sequence = write_acknowledgement.sequence().clone();
				let src_port_id = write_acknowledgement.src_port_id().clone();
				let src_channel_id = write_acknowledgement.src_channel_id().clone();
				let dst_port_id = write_acknowledgement.dst_port_id().clone();
				let dst_channel_id = write_acknowledgement.dst_channel_id().clone();
				let acknowledgement = write_acknowledgement.acknowledgement().as_bytes().to_vec();
				let dst_connection_id = write_acknowledgement.dst_connection_id().clone();

				Ok(Event::<T>::WriteAcknowledgement {
					packet_data,
					timeout_height,
					timeout_timestamp,
					sequence,
					src_port_id,
					src_channel_id,
					dst_port_id,
					dst_channel_id,
					acknowledgement,
					dst_connection_id,
				})
			},
			RawIbcEvent::AcknowledgePacket(acknowledge_packet) => {
				let timeout_height = acknowledge_packet.timeout_height().clone();
				let timeout_timestamp = acknowledge_packet.timeout_timestamp().clone();
				let sequence = acknowledge_packet.sequence().clone();
				let src_port_id = acknowledge_packet.src_port_id().clone();
				let src_channel_id = acknowledge_packet.src_channel_id().clone();
				let dst_port_id = acknowledge_packet.dst_port_id().clone();
				let dst_channel_id = acknowledge_packet.dst_channel_id().clone();
				let channel_ordering = acknowledge_packet.channel_ordering().clone();
				let src_connection_id = acknowledge_packet.src_connection_id().clone();

				Ok(Event::<T>::AcknowledgePacket {
					timeout_height,
					timeout_timestamp,
					sequence,
					src_port_id,
					src_channel_id,
					dst_port_id,
					dst_channel_id,
					channel_ordering,
					src_connection_id,
				})
			},
			RawIbcEvent::TimeoutPacket(time_out_packet) => {
				let timeout_height = time_out_packet.timeout_height().clone();
				let timeout_timestamp = time_out_packet.timeout_timestamp().clone();
				let sequence = time_out_packet.sequence().clone();
				let src_port_id = time_out_packet.src_port_id().clone();
				let src_channel_id = time_out_packet.src_channel_id().clone();
				let dst_port_id = time_out_packet.dst_port_id().clone();
				let dst_channel_id = time_out_packet.dst_channel_id().clone();

				Ok(Event::<T>::TimeoutPacket {
					timeout_height,
					timeout_timestamp,
					sequence,
					src_port_id,
					src_channel_id,
					dst_port_id,
					dst_channel_id,
				})
			},
			RawIbcEvent::ChannelClosed(timeout_on_close_packet) => {
				let port_id = timeout_on_close_packet.port_id().clone();
				let channel_id = timeout_on_close_packet.channel_id().clone();
				let counterparty_port_id = timeout_on_close_packet.counterparty_port_id().clone();
				let maybe_counterparty_channel_id =
					timeout_on_close_packet.counterparty_channel_id().map(|value| value.clone());
				let connection_id = timeout_on_close_packet.connection_id().clone();
				let channel_ordering = timeout_on_close_packet.channel_ordering().clone();

				Ok(Event::<T>::ChannelClosed {
					port_id,
					channel_id,
					counterparty_port_id,
					maybe_counterparty_channel_id,
					connection_id,
					channel_ordering,
				})
			},
			RawIbcEvent::AppModule(app_module) => Ok(Event::<T>::AppModule(app_module)),
		}
	}
}
