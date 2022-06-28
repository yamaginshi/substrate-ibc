use super::*;
use sp_core::H256;

impl<T: Config> Pallet<T> {
	pub fn build_trie_inputs() -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error<T>> {
		todo!()
	}

	fn build_ibc_state_root() -> Result<H256, Error<T>> {
		todo!()
	}

	fn extract_ibc_state_root() -> Result<Vec<u8>, Error<T>> {
		todo!()
	}
}
