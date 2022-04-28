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

use crate::{alloc::borrow::ToOwned, ibc_core::routing::IbcModule};
use alloc::{
	borrow::{Borrow, Cow},
	collections::BTreeMap,
	sync::Arc,
};
use ibc::core::ics26_routing::context::{Module, ModuleId};
use scale_info::TypeInfo;

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: MockRouter,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = MockRouterBuilder::default()
			.add_route("ibcmodule".parse().unwrap(), IbcModule::default())
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: r }
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Default)]
pub struct MockRouterBuilder(MockRouter);

impl RouterBuilder for MockRouterBuilder {
	type Router = MockRouter;

	fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.0 .0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	fn build(self) -> Self::Router {
		self.0
	}
}

#[derive(Debug, Default, Clone)]
pub struct MockRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl ibc::core::ics26_routing::context::Router for MockRouter {
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		log::trace!("in routing: [get_route_mut]");

		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		log::trace!("in routing: [has_route]");
		self.0.get(module_id.borrow()).is_some()
	}
}
