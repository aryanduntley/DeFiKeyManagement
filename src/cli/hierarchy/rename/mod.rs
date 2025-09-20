pub mod rename_wallet_group;
pub mod rename_address_group;
pub mod rename_wallet;
pub mod rename_subwallet;
pub mod rename_standalone_wallet;

pub use rename_wallet_group::RenameWalletGroupArgs;
pub use rename_address_group::RenameAddressGroupArgs;
pub use rename_wallet::RenameWalletArgs;
pub use rename_subwallet::RenameSubwalletArgs;
pub use rename_standalone_wallet::RenameStandaloneWalletArgs;