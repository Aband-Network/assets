
pub trait GetMultiAssetInfo<AssetId, Metadata> {
    fn get_metadata(asset_id: AssetId) -> Option<Metadata>;
}

impl<AssetId, Metadata> GetMultiAssetInfo<AssetId, Metadata> for (){
    fn get_metadata(asset_id: AssetId) -> Option<Metadata> {
        None
    }
}