//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
mod sub_address_registry {
    use ink_lang as ink;
    use ink_prelude::string::String;
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
    }

    // The SubAuction result types.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SubAuction {
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
            self.artion = artion;

            Ok(())
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
