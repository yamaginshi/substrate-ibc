use crate::*;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as ICS20Error,
	},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement as GenericAcknowledgement,
			packet::Packet,
			Version,
		},
		ics05_port::capabilities::ChannelCapability,
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{Ics26Context, ModuleOutput, OnRecvPacketAck, RouterBuilder},
	},
	signer::Signer,
};

use crate::{alloc::borrow::ToOwned, context::Context};
use alloc::{
	borrow::{Borrow, Cow},
	collections::BTreeMap,
	sync::Arc,
};
use ibc::core::ics26_routing::context::{Module, ModuleId};
use scale_info::TypeInfo;

#[derive(Debug, Default)]
pub struct IbcModule;

impl Module for IbcModule {
	#[allow(clippy::too_many_arguments)]
	fn on_chan_open_init(
		&mut self,
		_output: &mut ModuleOutput,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_channel_cap: &ChannelCapability,
		_counterparty: &Counterparty,
		_version: &Version,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	#[allow(clippy::too_many_arguments)]
	fn on_chan_open_try(
		&mut self,
		_output: &mut ModuleOutput,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_channel_cap: &ChannelCapability,
		_counterparty: &Counterparty,
		_version: &Version,
		_counterparty_version: &Version,
	) -> Result<Version, Ics04Error> {
		todo!()
	}

	fn on_chan_open_ack(
		&mut self,
		_output: &mut ModuleOutput,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty_version: &Version,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_open_confirm(
		&mut self,
		_output: &mut ModuleOutput,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_close_init(
		&mut self,
		_output: &mut ModuleOutput,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_close_confirm(
		&mut self,
		_output: &mut ModuleOutput,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_recv_packet(
		&self,
		_output: &mut ModuleOutput,
		_packet: &Packet,
		_relayer: &Signer,
	) -> OnRecvPacketAck {
		OnRecvPacketAck::Nil(Box::new(|_| Ok(())))
	}

	fn on_acknowledgement_packet(
		&mut self,
		_output: &mut ModuleOutput,
		_packet: &Packet,
		_acknowledgement: &GenericAcknowledgement,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_timeout_packet(
		&mut self,
		_output: &mut ModuleOutput,
		_packet: &Packet,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		Ok(())
	}
}

impl<T: Config> Ics26Context for Context<T> {
	type Router = crate::context::MockRouter;

	fn router(&self) -> &Self::Router {
		log::trace!("in routing: [route]");
		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		log::trace!("in routing: [router_mut]");
		&mut self.router
	}
}
