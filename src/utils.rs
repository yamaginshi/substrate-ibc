use crate::Config;
use codec::Encode;
use scale_info::prelude::{fmt::Debug, format, string::String, vec::Vec};
use sp_std::vec;

use ibc::{
	applications::transfer::{error::Error as Ics20Error, VERSION},
	core::ics24_host::identifier::{ChannelId, PortId},
	signer::Signer,
};

pub(crate) const LOG_TARGET: &str = "runtime::pallet-ibc";

pub trait AssetIdAndNameProvider<AssetId> {
	type Err: Debug;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<AssetId, Self::Err>;

	fn try_get_asset_name(asset_id: AssetId) -> Result<Vec<u8>, Self::Err>;
}

pub fn host_height<T: Config>() -> u64 {
	let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
	let current_height: u64 = block_number.parse().unwrap_or_default();
	current_height
}

pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: &ChannelId,
) -> Result<Signer, Ics20Error> {
	let contents = format!("{}/{}", port_id, channel_id);
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	hex_string.parse::<Signer>().map_err(Ics20Error::signer)
}

pub fn offchain_key<T: Config>(channel_id: Vec<u8>, port_id: Vec<u8>) -> Vec<u8> {
	let pair = (T::INDEXING_PREFIX.to_vec(), channel_id, port_id);
	pair.encode()
}

/// Get trie key by applying the commitment prefix to the path and scale encoding the result
pub fn apply_prefix_and_encode(prefix: &[u8], path: Vec<String>) -> Vec<u8> {
	let mut key_path = vec![prefix];
	let path = path.iter().map(|p| p.as_bytes()).collect::<Vec<_>>();
	key_path.extend_from_slice(&path);
	key_path.encode()
}
