use alloc::vec::Vec;
pub trait AssetIdAndNameProvider<AssetId> {
	type Err;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<AssetId, Self::Err>;

	fn try_get_asset_name(asset_id: AssetId) -> Result<Vec<u8>, Self::Err>;
}
