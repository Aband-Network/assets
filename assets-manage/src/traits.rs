use crate::AssetMetadata;

pub trait GetMultiAssetInfo<AssetId, Metadata, AssetDetails> {
    fn get_asset_details(asset_id: AssetId) -> Option<AssetDetails>;

}

impl<AssetId, Metadata, AssetDetails> GetMultiAssetInfo<AssetId, Metadata, AssetDetails> for (){
    fn get_asset_details(asset_id: AssetId) -> Option<AssetDetails> {
        None
    }
}