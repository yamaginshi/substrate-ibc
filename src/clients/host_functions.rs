use crate::clients::state_machine::read_proof_check;
use ibc::{
	clients::{host_functions::HostFunctionsProvider, ics10_grandpa::error::Error as ICS10Error},
	core::{
		ics02_client::error::Error as ICS02Error, ics23_commitment::error::Error as ICS23Error,
	},
};
use scale_info::prelude::vec::Vec;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::StorageProof;

#[derive(Clone, Default)]
pub struct HostFunctions;

impl HostFunctionsProvider for HostFunctions {
	fn keccak_256(input: &[u8]) -> [u8; 32] {
		sp_io::hashing::keccak_256(input)
	}

	fn secp256k1_ecdsa_recover_compressed(
		signature: &[u8; 65],
		value: &[u8; 32],
	) -> Option<Vec<u8>> {
		sp_io::crypto::secp256k1_ecdsa_recover_compressed(signature, value)
			.ok()
			.map(|pub_key| pub_key.to_vec())
	}

	fn ed25519_verify(signature: &[u8; 64], msg: &[u8], pubkey: &[u8]) -> bool {
		let signature = sp_core::ed25519::Signature::from_raw(*signature);
		if let Ok(public_key) = sp_core::ed25519::Public::try_from(pubkey) {
			sp_io::crypto::ed25519_verify(&signature, msg, &public_key)
		} else {
			false
		}
	}

	fn verify_membership_trie_proof(
		root: &[u8; 32],
		proof: &[Vec<u8>],
		key: &[u8],
		value: &[u8],
	) -> Result<(), ICS02Error> {
		let root = H256::from_slice(root);
		let proof_value =
			read_proof_check::<BlakeTwo256>(root, StorageProof::new(proof.to_vec()), key)
				.map_err(|_| ICS02Error::read_proof_check())?
				.ok_or_else(ICS02Error::empty_proof)?;

		if proof_value == value {
			Ok(())
		} else {
			Err(ICS02Error::grandpa(ICS10Error::ics23_error(ICS23Error::verification_failure())))
		}
	}

	fn verify_non_membership_trie_proof(
		root: &[u8; 32],
		proof: &[Vec<u8>],
		key: &[u8],
	) -> Result<(), ICS02Error> {
		let root = H256::from_slice(root);
		let proof_value =
			read_proof_check::<BlakeTwo256>(root, StorageProof::new(proof.to_vec()), key)
				.map_err(|_| ICS02Error::read_proof_check())?;

		if proof_value.is_none() {
			Ok(())
		} else {
			Err(ICS02Error::grandpa(ICS10Error::ics23_error(ICS23Error::verification_failure())))
		}
	}

	fn sha256_digest(data: &[u8]) -> [u8; 32] {
		sp_io::hashing::sha2_256(data)
	}

	fn sha2_256(message: &[u8]) -> [u8; 32] {
		sp_io::hashing::sha2_256(message)
	}

	fn sha2_512(message: &[u8]) -> [u8; 64] {
		use sha2::Digest;
		let mut hasher = sha2::Sha512::new();
		hasher.update(message);
		let hash = hasher.finalize();
		let mut res = [0u8; 64];
		res.copy_from_slice(&hash);
		res
	}

	fn sha2_512_truncated(message: &[u8]) -> [u8; 32] {
		use sha2::Digest;
		let mut hasher = sha2::Sha512::new();
		hasher.update(message);
		let hash = hasher.finalize();
		let mut res = [0u8; 32];
		res.copy_from_slice(&hash[..32]);
		res
	}

	fn sha3_512(message: &[u8]) -> [u8; 64] {
		use sha3::Digest;
		let mut hasher = sha3::Sha3_512::new();
		hasher.update(message);
		let hash = hasher.finalize();
		let mut res = [0u8; 64];
		res.copy_from_slice(&hash);
		res
	}

	fn ripemd160(message: &[u8]) -> [u8; 20] {
		use ripemd::Digest;
		let mut hasher = ripemd::Ripemd160::new();
		hasher.update(message);
		let hash = hasher.finalize();
		let mut res = [0u8; 20];
		res.copy_from_slice(&hash);
		res
	}
}
