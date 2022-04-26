use super::*;

use crate::routing::Context;
use ibc::core::{
	ics05_port::{
		capabilities::{Capability, CapabilityName, PortCapability},
		context::{CapabilityKeeper, CapabilityReader, PortKeeper, PortReader},
		error::Error as Ics05Error,
	},
	ics24_host::identifier::PortId,
	ics26_routing::context::ModuleId,
};

impl<T: Config> CapabilityReader for Context<T> {
	/// Fetch a capability which was previously claimed by specified name
	fn get_capability(&self, name: &CapabilityName) -> Result<Capability, Ics05Error> {
		todo!()
	}

	/// Authenticate a given capability and name. Lookup the capability from the internal store and
	/// check against the provided name.
	fn authenticate_capability(
		&self,
		name: &CapabilityName,
		capability: &Capability,
	) -> Result<(), Ics05Error> {
		todo!()
	}
}

impl<T: Config> CapabilityKeeper for Context<T> {
	/// Create a new capability with the given name.
	/// Return an error if the capability was already taken.
	fn new_capability(&mut self, name: CapabilityName) -> Result<Capability, Ics05Error> {
		todo!()
	}

	/// Claim the specified capability using the specified name.
	/// Return an error if the capability was already taken.
	fn claim_capability(&mut self, name: CapabilityName, capability: Capability) {
		todo!()
	}

	/// Release a previously claimed or created capability
	fn release_capability(&mut self, name: CapabilityName, capability: Capability) {
		todo!()
	}
}

impl<T: Config> PortReader for Context<T> {
	/// Return the module_id along with the capability associated with a given port_id
	fn lookup_module_by_port(
		&self,
		port_id: &PortId,
	) -> Result<(ModuleId, PortCapability), Ics05Error> {
		log::trace!("in port: [lookup_module_by_port]");
		// todo
		let module_id = ModuleId::new("ibcmodule".to_string().into()).unwrap();
		Ok((module_id, Capability::new().into()))
	}
}

impl<T: Config> PortKeeper for Context<T> {}
