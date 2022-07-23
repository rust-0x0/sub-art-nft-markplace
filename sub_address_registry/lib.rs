//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_address_registry::{SubAddressRegistry,SubAddressRegistryRef};

#[cfg_attr(test, allow(dead_code))]
const INTERFACE_ID_ERC721: [u8; 4] = [0x80, 0xAC, 0x58, 0xCD];

use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
pub mod sub_address_registry {
    pub type TokenId = u128;

    use ink_lang as ink;
    use ink_prelude::string::String;
     use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubAddressRegistry {
        /// @notice Artion contract
        artion: AccountId,
        /// @notice Auction contract
        auction: AccountId,
        /// @notice Marketplace contract
        marketplace: AccountId,
        /// @notice BundleMarketplace contract
        bundle_marketplace: AccountId,
        /// @notice NFTFactory contract
        factory: AccountId,
        /// @notice NFTFactoryPrivate contract
        private_factory: AccountId,
        /// @notice ArtFactory contract
        art_factory: AccountId,
        /// @notice ArtFactoryPrivate contract
        private_art_factory: AccountId,
        /// @notice TokenRegistry contract
        token_registry: AccountId,
        /// @notice PriceFeed contract
        price_feed: AccountId,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwner,
NotERC721,
    }

    // The SubAddressRegistry result types.
    pub type Result<T> = core::result::Result<T, Error>;

/// Event emitted when a token ItemListed occurs.
    #[ink(event)]
    pub struct ItemListed {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
        pub quantity: u128,
        pub pay_token: AccountId,
        pub price_per_item: Balance,
        pub starting_time: u128,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ItemSold {
        #[ink(topic)]
        pub seller: AccountId,
        #[ink(topic)]
        pub buyer: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
        pub quantity: u128,
        pub pay_token: AccountId,
        pub unit_price: Balance,
        pub price_per_item: Balance,
    }

    /// Event emitted when a token ItemUpdated occurs.
    #[ink(event)]
    pub struct ItemUpdated {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
        pub pay_token: AccountId,
        pub new_price: Balance,
    }

    #[ink(event)]
    pub struct ItemCanceled {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
    }

    /// Event emitted when a token OfferCreated occurs.
    #[ink(event)]
    pub struct OfferCreated {
        #[ink(topic)]
        pub creator: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
        pub quantity: u128,
        pub pay_token: AccountId,
        pub price_per_item: Balance,
        pub deadline: u128,
    }
    /// Event emitted when a token OfferCanceled occurs.
    #[ink(event)]
    pub struct OfferCanceled {
        #[ink(topic)]
        pub creator: AccountId,
        #[ink(topic)]
        pub nft_address: AccountId,
        pub token_id: TokenId,
    }

    /// Event emitted when a token UpdatePlatformFee occurs.
    #[ink(event)]
    pub struct UpdatePlatformFee {
        pub platform_fee: Balance,
    }
    /// Event emitted when a token UpdatePlatformFeeRecipient occurs.
    #[ink(event)]
    pub struct UpdatePlatformFeeRecipient {
        pub fee_recipient: AccountId,
    }

    impl SubAddressRegistry {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
            })
        }

        /**
        @notice Update artion contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_artion(&mut self, artion: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            //   require(
            //             IERC165(_artion).supportsInterface(INTERFACE_ID_ERC721),
            //             "Not ERC721"
            //         );
            use crate::INTERFACE_ID_ERC721;
            ensure!(self.supports_interface_check(artion,INTERFACE_ID_ERC721), Error::NotERC721);
            self.artion = artion;

            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: [u8;4]) -> bool {
            // This is disabled during tests due to the use of `invoke_contract()` not being
            // supported (tests end up panicking).
            let mut ans = false;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let supports_interface_selector: [u8; 4] = [0xF2, 0x3A, 0x6E, 0x61];
                // If our recipient is a smnft contract we need to see if they accept or
                // reject this transfer. If they reject it we need to revert the call.
                let params = build_call::<Environment>()
                    .call_type(Call::new().callee(callee).gas_limit(5000))
                    .exec_input(
                        ExecutionInput::new(Selector::new(supports_interface_selector))
                            .push_arg(data),
                    )
                    .returns::<Vec<u8>>()
                    .params();

                match ink_env::invoke_contract(&params) {
                    Ok(v) => {
                        ink_env::debug_println!(
                            "Received return value \"{:?}\" from contract {:?}",
                            v,
                            data
                        );
                        ans = v == &data[..];
                        // assert_eq!(
                        //     v,
                        //     &ON_ERC_721_RECEIVED_SELECTOR[..],
                        //     "The recipient contract at {:?} does not accept token transfers.\n
                        //     Expected: {:?}, Got {:?}",
                        //     to,
                        //     ON_ERC_721_RECEIVED_SELECTOR,
                        //     v
                        // )
                    }
                    Err(e) => {
                        match e {
                            ink_env::Error::CodeNotFound | ink_env::Error::NotCallable => {
                                // Our recipient wasn't a smnft contract, so there's nothing more for
                                // us to do
                                ink_env::debug_println!(
                                    "Recipient at {:?} from is not a smnft contract ({:?})",
                                    callee,
                                    e
                                );
                            }
                            _ => {
                                // We got some sort of error from the call to our recipient smnft
                                // contract, and as such we must revert this call
                                // panic!("Got error \"{:?}\" while trying to call {:?}", e, from)
                            }
                        }
                    }
                }
            }
            ans
        }
        /**
        @notice Update Auction contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_auction(&mut self, auction: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.auction = auction;
            Ok(())
        }
        /**
        @notice Update Marketplace contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_marketplace(&mut self, marketplace: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);

            self.marketplace = marketplace;
            Ok(())
        }

        /**
        @notice Update BundleMarketplace contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_bundle_marketplace(&mut self, bundle_marketplace: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.bundle_marketplace = bundle_marketplace;
            Ok(())
        }

        /**
        @notice Update NFTFactory contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_nft_factory(&mut self, factory: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.factory = factory;
            Ok(())
        }

        /**
        @notice Update NFTFactoryPrivate contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_nft_factory_private(&mut self, private_factory: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.private_factory = private_factory;
            Ok(())
        }
        /**
        @notice Update ArtFactory contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_art_factory(&mut self, art_factory: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.art_factory = art_factory;
            Ok(())
        }

        /**
        @notice Update ArtFactoryPrivate contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_art_factory_private(&mut self, private_art_factory: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.private_art_factory = private_art_factory;
            Ok(())
        }
        /**
        @notice Update token registry contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_token_registry(&mut self, token_registry: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.token_registry = token_registry;
            Ok(())
        }
        /**
        @notice Update price feed contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_price_feed(&mut self, price_feed: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.price_feed = price_feed;
            Ok(())
        }
        #[ink(message)]
        pub fn artion(& self ) -> AccountId {

            self.artion 
        }
  #[ink(message)]
        pub fn auction(&self)->AccountId {
            self.auction 
        }
        /**
        @notice Update Marketplace contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn marketplace(&self)->AccountId {
            self.marketplace
        }

        /**
        @notice Update BundleMarketplace contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn bundle_marketplace(&self)->AccountId {
            self.bundle_marketplace 
        }

        /**
        @notice Update NFTFactory contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn nft_factory(&self)->AccountId {
            self.factory 
        }

        /**
        @notice Update NFTFactoryPrivate contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn nft_factory_private(&self)->AccountId {
            self.private_factory 
        }
        /**
        @notice Update ArtFactory contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn art_factory(&self)->AccountId {
            self.art_factory 
        }

        /**
        @notice Update ArtFactoryPrivate contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn art_factory_private(&self)->AccountId {
            self.private_art_factory 
        }
        /**
        @notice Update token registry contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn token_registry(&self)->AccountId {
            self.token_registry 
        }
        /**
        @notice Update price feed contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn price_feed(&self)->AccountId {
            self.price_feed
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        fn set_caller(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }
    }
}
