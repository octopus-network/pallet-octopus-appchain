use sp_runtime::KeyTypeId;

/// Means for interacting with a specialized version of the `session` trait.
pub trait SessionInterface<AccountId>: frame_system::Config {
	fn same_validator(id: KeyTypeId, key_data: &[u8], validator: AccountId) -> bool;
}
