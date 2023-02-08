use super::*;
use codec::{Encode, Decode};
use sp_runtime::traits::{AccountIdConversion};
use sp_runtime::TypeId;

#[derive(Encode, Decode)]
pub struct AssetId<Id>(pub Id);

impl<CurrencyId> TypeId for AssetId<CurrencyId> {
	const TYPE_ID: [u8; 4] = *b"asst";
}

impl<Id> From<Id> for AssetId<Id> {
	fn from(value: Id) -> Self {
		Self(value)
	}
}
