use codec::Codec;
use hash_db::{HashDB, Hasher, EMPTY_PREFIX};
use scale_info::prelude::vec::Vec;
use sp_std::prelude::*;
use sp_trie::{read_trie_value, LayoutV0, MemoryDB, StorageProof, TrieError};

/// Create in-memory storage of proof check backend.
fn create_proof_check_backend_storage<H>(proof: StorageProof) -> MemoryDB<H>
where
	H: Hasher,
{
	let mut db = MemoryDB::default();
	for item in proof.iter_nodes() {
		db.insert(EMPTY_PREFIX, &item);
	}
	db
}

/// Check storage read proof, generated by `prove_read` call.
pub fn read_proof_check<H>(
	root: H::Out,
	proof: StorageProof,
	key: &[u8],
) -> Result<Option<Vec<u8>>, Box<TrieError<LayoutV0<H>>>>
where
	H: Hasher,
	H::Out: Ord + Codec,
{
	let db = create_proof_check_backend_storage::<H>(proof);

	if db.contains(&root, EMPTY_PREFIX) {
		read_trie_value::<LayoutV0<H>, _>(&db, &root, key)
	} else {
		Err(Box::new(TrieError::<LayoutV0<H>>::InvalidStateRoot(root)))
	}
}

#[test]
fn create_proof_check_backend_works() {
	use codec::{Decode, Encode};
	use sp_runtime::traits::BlakeTwo256;

	let key: Vec<u8> = vec![
		101, 204, 242, 3, 105, 192, 221, 218, 216, 45, 16, 3, 82, 58, 196, 142, 47, 99, 99, 80, 87,
		249, 175, 51, 232, 228, 192, 128, 156, 156, 52, 197, 83, 68, 105, 160, 53, 236, 227, 28,
		226, 64, 250, 25, 185, 226, 109, 66, 48, 99, 111, 110, 110, 101, 99, 116, 105, 111, 110,
		45, 48,
	];
	let state_root: [u8; 32] = [
		249, 132, 94, 43, 77, 149, 90, 95, 219, 102, 214, 18, 162, 227, 15, 122, 192, 216, 194, 6,
		72, 47, 16, 134, 28, 241, 21, 141, 224, 243, 92, 16,
	];
	let proof: Vec<Vec<u8>> = vec![
		vec![
			127, 26, 15, 99, 99, 80, 87, 249, 175, 51, 232, 228, 192, 128, 156, 156, 52, 197, 83,
			68, 105, 160, 53, 236, 227, 28, 226, 64, 250, 25, 185, 226, 109, 66, 48, 99, 111, 110,
			110, 101, 99, 116, 105, 111, 110, 45, 48, 57, 1, 49, 1, 10, 12, 49, 48, 45, 103, 114,
			97, 110, 100, 112, 97, 45, 48, 18, 35, 10, 1, 49, 18, 13, 79, 82, 68, 69, 82, 95, 79,
			82, 68, 69, 82, 69, 68, 18, 15, 79, 82, 68, 69, 82, 95, 85, 78, 79, 82, 68, 69, 82, 69,
			68, 24, 1, 34, 21, 10, 12, 49, 48, 45, 103, 114, 97, 110, 100, 112, 97, 45, 48, 26, 5,
			10, 3, 105, 98, 99,
		],
		vec![
			128, 40, 1, 128, 62, 136, 130, 206, 114, 209, 251, 2, 186, 214, 149, 4, 63, 31, 153,
			58, 78, 36, 37, 144, 250, 187, 101, 55, 80, 104, 213, 83, 252, 255, 184, 85, 128, 132,
			241, 93, 23, 185, 251, 174, 180, 147, 75, 149, 134, 104, 22, 97, 192, 53, 229, 94, 19,
			188, 247, 62, 196, 26, 150, 151, 186, 14, 110, 61, 146, 128, 219, 71, 190, 229, 185,
			138, 122, 36, 3, 54, 115, 167, 70, 54, 66, 179, 53, 234, 152, 26, 170, 34, 129, 136,
			36, 195, 239, 98, 8, 211, 228, 128,
		],
		vec![
			128, 111, 188, 128, 64, 217, 27, 149, 112, 195, 19, 90, 23, 212, 111, 91, 65, 160, 2,
			74, 13, 125, 29, 53, 153, 246, 232, 182, 93, 73, 122, 71, 192, 2, 251, 49, 128, 204,
			30, 119, 130, 105, 6, 200, 50, 10, 12, 201, 156, 57, 97, 59, 20, 71, 166, 67, 42, 181,
			1, 103, 250, 155, 150, 0, 42, 86, 170, 201, 77, 128, 189, 30, 112, 39, 133, 170, 78,
			189, 185, 179, 100, 132, 66, 222, 77, 107, 244, 66, 102, 12, 229, 153, 11, 70, 62, 26,
			85, 83, 111, 112, 129, 35, 128, 171, 201, 157, 222, 121, 232, 162, 189, 172, 28, 190,
			71, 6, 247, 59, 80, 207, 31, 99, 123, 167, 31, 167, 25, 48, 19, 118, 238, 113, 28, 41,
			137, 128, 0, 45, 215, 36, 161, 66, 66, 16, 43, 130, 200, 196, 211, 0, 246, 209, 35,
			161, 72, 5, 138, 124, 126, 42, 210, 175, 64, 146, 203, 237, 200, 129, 128, 121, 61, 78,
			155, 72, 65, 137, 93, 79, 36, 14, 50, 63, 233, 226, 97, 201, 219, 213, 76, 98, 67, 243,
			196, 139, 122, 140, 105, 124, 178, 141, 42, 128, 244, 36, 179, 113, 248, 139, 40, 70,
			59, 42, 233, 127, 220, 187, 92, 241, 225, 254, 131, 206, 204, 221, 109, 11, 66, 68, 83,
			35, 0, 209, 47, 99, 128, 254, 57, 63, 227, 147, 47, 90, 19, 137, 103, 67, 134, 129, 10,
			176, 38, 10, 143, 50, 97, 169, 30, 8, 100, 132, 157, 183, 66, 235, 98, 203, 121, 128,
			74, 123, 98, 46, 8, 5, 33, 149, 214, 35, 217, 149, 77, 159, 132, 146, 29, 152, 104,
			147, 21, 227, 100, 2, 28, 142, 127, 140, 90, 255, 189, 216, 128, 49, 34, 27, 125, 242,
			195, 135, 152, 134, 172, 1, 161, 250, 77, 15, 195, 48, 136, 173, 253, 119, 192, 183,
			107, 11, 157, 21, 184, 85, 64, 209, 97, 128, 88, 110, 207, 241, 15, 220, 243, 170, 211,
			153, 6, 15, 96, 192, 120, 121, 17, 179, 88, 12, 115, 127, 12, 152, 24, 50, 226, 4, 162,
			127, 129, 75,
		],
		vec![
			158, 204, 242, 3, 105, 192, 221, 218, 216, 45, 16, 3, 82, 58, 196, 142, 253, 162, 104,
			95, 10, 188, 245, 27, 222, 65, 138, 14, 86, 107, 122, 75, 66, 31, 59, 242, 32, 1, 0, 0,
			0, 0, 0, 0, 0, 128, 75, 86, 212, 81, 195, 153, 61, 188, 55, 220, 238, 30, 206, 67, 78,
			72, 240, 242, 116, 251, 7, 27, 193, 75, 95, 187, 84, 73, 34, 190, 137, 208, 128, 53, 0,
			209, 82, 92, 162, 85, 27, 254, 131, 36, 81, 173, 239, 243, 238, 49, 9, 2, 20, 91, 211,
			1, 12, 18, 197, 28, 156, 97, 206, 52, 50, 80, 95, 14, 123, 144, 18, 9, 107, 65, 196,
			235, 58, 175, 148, 127, 110, 164, 41, 8, 0, 0, 128, 241, 43, 109, 237, 3, 5, 124, 173,
			250, 112, 127, 91, 1, 123, 87, 229, 10, 229, 181, 74, 26, 111, 51, 183, 174, 38, 73,
			15, 3, 24, 138, 3, 128, 199, 154, 161, 103, 129, 23, 48, 12, 194, 6, 36, 222, 124, 206,
			174, 206, 159, 14, 64, 60, 96, 192, 1, 195, 178, 111, 142, 201, 248, 227, 54, 202, 104,
			95, 13, 175, 218, 65, 33, 225, 150, 51, 237, 160, 123, 37, 248, 10, 100, 93, 32, 1, 0,
			0, 0, 0, 0, 0, 0, 128, 147, 149, 247, 200, 11, 49, 136, 240, 87, 167, 188, 28, 17, 119,
			200, 245, 212, 141, 240, 156, 204, 25, 230, 44, 176, 3, 231, 13, 205, 52, 36, 123, 128,
			187, 56, 234, 165, 114, 210, 102, 191, 164, 89, 187, 203, 112, 111, 240, 116, 113, 223,
			92, 254, 8, 249, 150, 58, 141, 161, 16, 223, 5, 154, 53, 1, 88, 95, 13, 220, 13, 194,
			118, 101, 4, 18, 67, 243, 210, 50, 39, 78, 175, 183, 16, 12, 16, 210, 1,
		],
	];
	let storage_value = read_proof_check::<BlakeTwo256>(
		sp_core::H256::from(state_root),
		StorageProof::new(proof),
		&key,
	)
	.unwrap()
	.unwrap();
	let mut _storage_value: Vec<u8> = vec![
		49, 1, 10, 12, 49, 48, 45, 103, 114, 97, 110, 100, 112, 97, 45, 48, 18, 35, 10, 1, 49, 18,
		13, 79, 82, 68, 69, 82, 95, 79, 82, 68, 69, 82, 69, 68, 18, 15, 79, 82, 68, 69, 82, 95, 85,
		78, 79, 82, 68, 69, 82, 69, 68, 24, 1, 34, 21, 10, 12, 49, 48, 45, 103, 114, 97, 110, 100,
		112, 97, 45, 48, 26, 5, 10, 3, 105, 98, 99,
	];
	assert_eq!(storage_value, _storage_value);

	let storage_value = <Vec<u8>>::decode(&mut &_storage_value[..]).unwrap();
	_storage_value.remove(0);
	_storage_value.remove(0);
	assert_eq!(storage_value, _storage_value);
}
