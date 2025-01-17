use crate::{context::Context, utils::host_height, Config, REVISION_NUMBER};
pub use alloc::{
	format,
	string::{String, ToString},
};
use ibc::{
	core::{
		ics02_client::{client_state::ClientState, context::ClientReader, header::Header},
		ics24_host::identifier::ClientId,
		ics26_routing::handler::{deliver, MsgReceipt},
	},
	events::IbcEvent,
	relayer::ics18_relayer::{context::RelayerContext, error::RelayerError},
	signer::Signer,
	Height,
};
use ibc_proto::google::protobuf::Any;
use scale_info::prelude::{vec, vec::Vec};
use sp_std::boxed::Box;

impl<T: Config> RelayerContext for Context<T> {
	fn query_latest_height(&self) -> Result<Height, RelayerError> {
		let revision_height = host_height::<T>();
		Ok(Height::new(REVISION_NUMBER, revision_height).expect(&REVISION_NUMBER.to_string()))
	}

	fn query_client_full_state(&self, client_id: &ClientId) -> Option<Box<dyn ClientState>> {
		// Forward call to Ics2.
		ClientReader::client_state(self, client_id).ok()
	}

	fn query_latest_header(&self) -> Option<Box<dyn Header>> {
		todo!()
	}

	fn send(&mut self, msgs: Vec<Any>) -> Result<Vec<IbcEvent>, RelayerError> {
		let mut all_events = vec![];
		for msg in msgs {
			let MsgReceipt { mut events, .. } =
				deliver(self, msg).map_err(RelayerError::TransactionFailed)?;
			all_events.append(&mut events);
		}
		Ok(all_events)
	}

	fn signer(&self) -> Signer {
		"0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
	}
}
