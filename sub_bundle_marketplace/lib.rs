//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_bundle_marketplace::{SubBundleMarketplace,SubBundleMarketplaceRef};

use ink_lang as ink;

#[cfg_attr(test, allow(dead_code))]
const INTERFACE_ID_ERC721: [u8; 4] = [0x80, 0xAC, 0x58, 0xCD];

const INTERFACE_ID_ERC1155: [u8; 4] = [0xD9, 0xB6, 0x7A, 0x26];

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
mod sub_bundle_marketplace {
    use ink_lang as ink;
 use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
 use ink_prelude::collections::BTreeSet;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };

    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u128;

    #[derive( Default,scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Listing {
        nfts: Vec<AccountId>,
        token_ids: Vec<TokenId>,
        pub quantities: Vec<u128>,
        pub pay_token: AccountId,
        pub price: Balance,
        pub starting_time: u128,
    }

    #[derive( Default,scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Offer {
        pub pay_token: AccountId,
        pub price: Balance,
        pub deadline: u128,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubBundleMarketplace {
        listings: Mapping<(AccountId, String), Listing>,
        owners: Mapping<String, AccountId>,
        bundle_ids_per_item: Mapping<(AccountId, TokenId), BTreeSet<String>>,
        nft_indices: Mapping<(String, AccountId, u128), u128>,
        bundle_ids: Mapping<String, String>,
        offers: Mapping<(String, AccountId), Offer>,

        /// @notice Address registry
        address_registry: AccountId,
        /// # note Platform fee
        platform_fee: Balance,
        /// # note Platform fee receipient
        fee_recipient: AccountId,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidPayToken,
        NotOwningItem,
        InvalidNFTAddress,
        AlreadyListed,
        ItemNotApproved,
        MustHoldEnoughNFTs,
        NotListedItem,
        ItemNotBuyable,
        InsufficientBalanceOrNotApproved,
        OnlyOwner,
        OfferAlreadyCreated,
        CannotPlaceAnOfferIfAuctionIsGoingOn,
        InvalidExpiration,
        OfferNotExistsOrExpired,
        InvalidRoyalty,
        RoyaltyAlreadySet,
        InvalidCreatorAddress,
        SenderMustBeBundleMarketplace,
InvalidData,
InvalidId,
InsufficientFunds,
FeeTransferFailed,
OwnerFeeTransferFailed,
FailedToSendTheOwnerFeeTransferFailed,
    }

    // The SubBundleMarketplace result types.
    pub type Result<T> = core::result::Result<T, Error>;
    /// Event emitted when a token ItemListed occurs.
    #[ink(event)]
    pub struct ItemListed {
        #[ink(topic)]
        owner: AccountId,
        bundle_id: String,
        pay_token: AccountId,
        price: Balance,
        starting_time: u128,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ItemSold {
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        buyer: AccountId,
        bundle_id: String,
        pay_token: AccountId,
        unit_price: Balance,
        price: Balance,
    }

    /// Event emitted when a token ItemUpdated occurs.
    #[ink(event)]
    pub struct ItemUpdated {
        #[ink(topic)]
        owner: AccountId,
        bundle_id: String,
        nft: Vec<AccountId>,
        token_id: Vec<TokenId>,
        quantity: Vec<u128>,
        pay_token: AccountId,
        new_price: Balance,
    }

    #[ink(event)]
    pub struct ItemCanceled {
        #[ink(topic)]
        owner: AccountId,
        bundle_id: String,
    }

    /// Event emitted when a token OfferCreated occurs.
    #[ink(event)]
    pub struct OfferCreated {
        #[ink(topic)]
        creator: AccountId,
        bundle_id: String,
        pay_token: AccountId,
        price: Balance,
        deadline: u128,
    }
    /// Event emitted when a token OfferCanceled occurs.
    #[ink(event)]
    pub struct OfferCanceled {
        #[ink(topic)]
        creator: AccountId,
        bundle_id: String,
    }

    /// Event emitted when a token UpdatePlatformFee occurs.
    #[ink(event)]
    pub struct UpdatePlatformFee {
        platform_fee: Balance,
    }
    /// Event emitted when a token UpdatePlatformFeeRecipient occurs.
    #[ink(event)]
    pub struct UpdatePlatformFeeRecipient {
        fee_recipient: AccountId,
    }

    impl SubBundleMarketplace {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(fee_recipient: AccountId, platform_fee: Balance) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.fee_recipient = fee_recipient;
                contract.platform_fee = platform_fee;
            })
        }

        /// @notice Method for get NFT bundle listing
        /// @param _owner Owner address
        /// @param _bundleID Bundle ID
        #[ink(message)]
        #[cfg_attr(test, allow(unused_variables))]
        pub fn get_listing(
            &self,
            owner: AccountId,
            bundle_id: String,
        ) -> (Vec<AccountId>, Vec<TokenId>, Vec<u128>, Balance, u128) {
              let _bundle_id=  self.get_bundle_id(&bundle_id);
            let listing = self.listings.get(&(owner, _bundle_id)).unwrap();
            (
                listing.nfts,
                listing.token_ids,
                listing.quantities,
                listing.price,
                listing.starting_time,
            )
        }

        /// @notice Method for listing NFT bundle
        /// @param _bundleID Bundle ID
        /// @param _nftAddresses Addresses of NFT contract
        /// @param _tokenIds Token IDs of NFT
        /// @param _quantities token amounts to list (needed for ERC-1155 NFTs, set as 1 for ERC-721)
        /// @param _price sale price for bundle
        /// @param _startingTime scheduling for a future sale
        #[ink(message)]
        #[cfg_attr(test, allow(unused_variables))]
        pub fn list_item(
            &mut self,
            bundle_id:String,
            nft_addresses: Vec<AccountId>,
            token_ids: Vec<TokenId>,
            quantities:Vec<u128>,
            pay_token: AccountId,
            price: Balance,
            starting_time: u128,
        ) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);
            self.bundle_ids.insert(&_bundle_id, &bundle_id);
            ensure!(
                nft_addresses.len() == token_ids.len() && quantities.len() == token_ids.len(),
                Error::InvalidData
            );
            let owner = self.owners.get(&_bundle_id).unwrap();

            let mut listing = self
                .listings
                .get(&(self.env().caller(), _bundle_id.clone()))
                .unwrap();
            ensure!(
                owner == AccountId::from([0x0; 32])
                    || (owner == self.env().caller() && listing.price == 0),
                Error::AlreadyListed
            );

            // #[cfg(not(test))]
            // {
            //     ensure!(
            //         AccountId::from([0x0; 32]) != self.address_registry,
            //         Error::InvalidPayToken
            //     );
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     if pay_token != AccountId::from([0x0; 32]) {
            //         ensure!(
            //             AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
            //             Error::InvalidPayToken
            //         );
            //         let token_registry_instance: TokenRegistry =
            //             ink_env::call::FromAccountId::from_account_id(
            //                 address_registry_instance.token_registry(),
            //             );
            //         ensure!(
            //             token_registry_instance.enabled(pay_token),
            //             Error::InvalidPayToken,
            //         );
            //     }
            // }
            listing.nfts.clear();
            listing.token_ids.clear();
            listing.quantities.clear();
            for (i, &nft_address) in nft_addresses.iter().enumerate() {
                let token_id = token_ids[i];
                let quantity = quantities[i];

                // #[cfg(not(test))]
                // {
                //     if self.supports_interface_check(nft_address, INTERFACE_ID_ERC721) {
                //         use erc721::Erc721;
                //         let erc721_instance: Erc721 =
                //             ink_env::call::FromAccountId::from_account_id(nft_address);
                //         ensure!(
                //             self.env().caller() == erc721_instance.owner_of(token_id),
                //             Error::NotOwningItem
                //         );
                //         ensure!(
                //             erc721_instance
                //                 .is_approved_for_all(self.env().caller(), self.env().account_id()),
                //             Error::ItemNotApproved
                //         );
                //         listing.quantities.push(1);
                //     } else if self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155) {
                //         use erc1155::Erc1155;
                //         let erc1155_instance: Erc1155 =
                //             ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //         ensure!(
                //             quantity <= erc1155_instance.balance_of(self.env().caller(), token_id),
                //             Error::MustHoldEnoughNFTs
                //         );
                //         ensure!(
                //             erc1155_instance
                //                 .is_approved_for_all(self.env().caller(), self.env().account_id()),
                //             Error::ItemNotApproved
                //         );
                //     } else {
                //         ensure!(false, Error::InvalidNFTAddress);
                //     }
                // }
                listing.nfts.push(nft_address);
                listing.token_ids.push(token_id);
                let mut items = self
                    .bundle_ids_per_item
                    .get(&(nft_address, token_id))
                    .unwrap();
                items.insert(_bundle_id.clone());
                self.bundle_ids_per_item
                    .insert(&(nft_address, token_id), &items);
                self.nft_indices
                    .insert(&(_bundle_id.clone(), nft_address, token_id), &(i as u128));
            }
            listing.pay_token = pay_token;
            listing.price = price;
            listing.starting_time = starting_time;
            self.listings.insert(&(self.env().caller(), _bundle_id.clone()), &listing);
            self.owners.insert(&_bundle_id, &self.env().caller());

            self.env().emit_event(ItemListed {
                owner: self.env().caller(),
                bundle_id,
                pay_token,
                price,
                starting_time,
            });
            Ok(())
        }

        /// @notice Method for canceling listed NFT bundle
        #[ink(message)]
        pub fn cancel_listing(&mut self, bundle_id: String) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);

            let listing = self
                .listings
                .get(&(self.env().caller(), _bundle_id))
                .unwrap();
            ensure!(listing.price > 0, Error::NotListedItem);
            self._cancel_listing(self.env().caller(), bundle_id)?;
            Ok(())
        }
        fn _cancel_listing(&mut self, owner: AccountId, bundle_id: String) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);

            let listing = self.listings.get(&(owner, _bundle_id.clone())).unwrap();
            for (i, &nft_address) in listing.nfts.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let mut items = self
                    .bundle_ids_per_item
                    .get(&(nft_address, token_id))
                    .unwrap();
                items.remove(&_bundle_id);
                self.bundle_ids_per_item
                    .insert(&(nft_address, token_id), &items);
                self.nft_indices
                    .remove(&(_bundle_id.clone(), nft_address, token_id));
            }

            self.listings.remove(&(owner, _bundle_id.clone()));
            self.owners.remove(&_bundle_id);
            self.bundle_ids.remove(&_bundle_id);
            self.env().emit_event(ItemCanceled { owner, bundle_id });
            Ok(())
        }

        /// @notice Method for updating listed NFT bundle
        /// @param _bundleID Bundle ID
        /// @param _newPrice New sale price for bundle
        #[ink(message)]
        pub fn update_listing(
            &mut self,
            bundle_id: String,
            pay_token: AccountId,
            new_price: Balance,
        ) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);
            let mut listing = self
                .listings
                .get(&(self.env().caller(), _bundle_id.clone()))
                .unwrap();
            ensure!(listing.price > 0, Error::NotListedItem);

            self.valid_pay_token(pay_token)?;

            listing.pay_token = pay_token;
            listing.price = new_price;
            self.listings
                .insert(&(self.env().caller(), _bundle_id.clone()), &listing);
            self.env().emit_event(ItemUpdated {
                owner: self.env().caller(),
                bundle_id,
                nft:listing.nfts,
                token_id:listing.token_ids,
                quantity:listing.quantities,
                pay_token,
                new_price,
            });
            Ok(())
        }

        /// @notice Method for buying listed NFT bundle
        /// @param _bundleID Bundle ID
        #[ink(message)]
        pub fn buy_item(&mut self, bundle_id: String, pay_token: AccountId) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&&bundle_id);

            let owner = self.owners.get(&_bundle_id).unwrap();
            ensure!(owner != AccountId::from([0x0; 32]), Error::InvalidId);

            let listing = self.listings.get(&(owner, _bundle_id)).unwrap();
            ensure!(listing.pay_token == pay_token, Error::InvalidPayToken);

            self._buy_item(bundle_id, pay_token)?;
            Ok(())
        }
        fn _buy_item(&mut self, bundle_id: String, pay_token: AccountId) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);
            let owner = self.owners.get(&_bundle_id).unwrap();
            let mut listing = self.listings.get(&(owner, _bundle_id.clone())).unwrap();
            ensure!(listing.price > 0, Error::NotListedItem);

            for (i, &nft_address) in listing.nfts.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                // #[cfg(not(test))]
                // {
                //     if self.supports_interface_check(nft_address, INTERFACE_ID_ERC721) {
                //         use erc721::Erc721;
                //         let erc721_instance: Erc721 =
                //             ink_env::call::FromAccountId::from_account_id(nft_address);
                //         ensure!(
                //             self.env().caller() == erc721_instance.owner_of(token_id),
                //             Error::NotOwningItem
                //         );
                //         ensure!(
                //             erc721_instance
                //                 .is_approved_for_all(self.env().caller(), self.env().account_id()),
                //             Error::ItemNotApproved
                //         );
                //     } else if self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155) {
                //         use erc1155::Erc1155;
                //         let erc1155_instance: Erc1155 =
                //             ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //         ensure!(
                //             quantity <= erc1155_instance.balance_of(owner, token_id),
                //             Error::MustHoldEnoughNFTs
                //         );
                //     } else {
                //         ensure!(false, Error::InvalidNFTAddress);
                //     }
                // }
            }

            ensure!(
                self.get_now() >= listing.starting_time,
                Error::ItemNotBuyable
            );

            let price = listing.price;
            let mut fee_amount = price * self.platform_fee / 1000;
            if pay_token == AccountId::from([0x0; 32]) {
                // Send platform fee
                ensure!(fee_amount <= self.env().balance(), Error::InsufficientFunds);
                ensure!(
                    self.env().transfer(self.fee_recipient, fee_amount).is_ok(),
                    Error::FeeTransferFailed
                );
                ensure!(
                    self.env().transfer(owner, price - fee_amount).is_ok(),
                    Error::OwnerFeeTransferFailed
                );
            } else {
                // #[cfg(not(test))]
                // {
                //     use erc20::Erc20;
                //     let erc20_instance: Erc20 =
                //         ink_env::call::FromAccountId::from_account_id(auction.pay_token);
                //     ensure!(
                //         erc20_instance
                //             .transfer_from(self.env().caller(), fee_recipient, fee_amount)
                //             .is_ok(),
                //         Error::FailedToSendTheOwnerFeeTransferFailed
                //     );
                //     ensure!(
                //         erc20_instance
                //             .transfer_from(self.env().caller(), owner, price - fee_amount)
                //             .is_ok(),
                //         Error::FailedToSendTheOwnerFeeTransferFailed
                //     );
                // }
            }

            for (i, &nft_address) in listing.nfts.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                // #[cfg(not(test))]
                // {
                //     if self.supports_interface_check(nft_address, INTERFACE_ID_ERC721) {
                //         use erc721::Erc721;
                //         let erc721_instance: Erc721 =
                //             ink_env::call::FromAccountId::from_account_id(nft_address);
                //         ensure!(
                //             erc721_instance
                //                 .transfer_from(owner, self.env().caller(), token_id)
                //                 .is_ok(),
                //             Error::NotOwningItem
                //         );
                //     } else if self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155) {
                //         use erc1155::Erc1155;
                //         let erc1155_instance: Erc1155 =
                //             ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //         ensure!(
                //             erc1155_instance.transfer_from(
                //                 owner,
                //                 self.env().caller(),
                //                 token_id,
                //                 listing.quantity,
                //                 Vec::new()
                //             ),
                //             Error::NotOwningItem
                //         );
                //     }
                //     use address_registry::AddressRegistry;
                //     let address_registry_instance: AddressRegistry =
                //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //     ensure!(
                //         AccountId::from([0x0; 32])
                //             == address_registry_instance.bundle_marketplace(),
                //         Error::InvalidPayToken
                //     );
                //     let marketplace_instance: BundleMarketplace =
                //         ink_env::call::FromAccountId::from_account_id(
                //             address_registry_instance.marketplace(),
                //         );
                //     marketplace_instance.validate_item_sold(
                //         nft_address,
                //         token_id,
                //         owner,
                //         self.env().caller(),
                //     );
                // }
            }
            self.listings.remove(&(owner, _bundle_id.clone()));
            listing.price = 0;
            self.listings
                .insert(&(self.env().caller(), _bundle_id.clone()), &listing);
            self.owners.insert(&_bundle_id, &self.env().caller());
            self.offers.remove(&(_bundle_id, self.env().caller()));
            self.env().emit_event(ItemSold {
                seller: owner,
                buyer: self.env().caller(),
                bundle_id:bundle_id.clone(),
                pay_token,
                unit_price: self.get_price(pay_token),
                price,
            });
            self.env().emit_event(OfferCanceled {
                creator: self.env().caller(),
                bundle_id,
            });
            Ok(())
        }
        /// @notice Method for offering bundle item
        /// @param _bundleID Bundle ID
        /// @param _payToken Paying token
        /// @param _price Price
        /// @param _deadline Offer expiration
        #[ink(message)]
        pub fn create_offer(
            &mut self,
            bundle_id: String,
            pay_token: AccountId,
            price: Balance,
            deadline: u128,
        ) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);
            let owner = self.owners.get(&_bundle_id).unwrap();
            ensure!(AccountId::from([0x0; 32]) != owner, Error::InvalidId);
            ensure!(deadline > self.get_now(), Error::InvalidExpiration);
            ensure!(price > 0, Error::InvalidExpiration);
            let offer = self.offers.get(&(_bundle_id.clone(), self.env().caller())).unwrap();
            ensure!(offer.deadline <= self.get_now(), Error::OfferAlreadyCreated);

            self.offers.insert(
                &(_bundle_id.clone(), self.env().caller()),
                &Offer {
                    pay_token,
                    price,
                    deadline,
                },
            );
            self.env().emit_event(OfferCreated {
                creator: self.env().caller(),
                bundle_id,
                pay_token,
                price,
                deadline,
            });
            Ok(())
        }
        /// @notice Method for canceling the offer
        /// @param _bundleID Bundle ID
        #[ink(message)]
        pub fn cancel_offer(&mut self, bundle_id: String) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);

            let offer = self.offers.get(&(_bundle_id.clone(), self.env().caller())).unwrap();
            ensure!(
                offer.deadline > self.get_now(),
                Error::OfferNotExistsOrExpired
            );
            self.offers.remove(&(_bundle_id.clone(), self.env().caller()));
            self.env().emit_event(OfferCanceled {
                creator: self.env().caller(),
                bundle_id,
            });
            Ok(())
        }

        /// @notice Method for accepting the offer
        /// @param _bundleID Bundle ID
        /// @param _creator Offer creator address
        #[ink(message)]
        pub fn accept_offer(&mut self, bundle_id: String, creator: AccountId) -> Result<()> {
            let _bundle_id = self.get_bundle_id(&bundle_id);
            let owner = self.owners.get(&_bundle_id).unwrap();
            ensure!(owner == self.env().caller(), Error::NotOwningItem);
            let offer = self.offers.get(&(_bundle_id.clone(), creator)).unwrap();
            ensure!(
                offer.deadline > self.get_now(),
                Error::OfferNotExistsOrExpired
            );

            let price = offer.price;
            let mut fee_amount = price * self.platform_fee / 1000;

            // #[cfg(not(test))]
            // {
            //     use erc20::Erc20;
            //     let erc20_instance: Erc20 =
            //         ink_env::call::FromAccountId::from_account_id(offer.pay_token);
            //     let result = erc20_instance.transfer_from(creator, fee_amount, fee_amount);
            //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
            //     let result =
            //         erc20_instance.transfer_from(creator, self.env().caller(), price - fee_amount);
            //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
            // }

            let mut listing = self
                .listings
                .get(&(self.env().caller(), _bundle_id.clone()))
                .unwrap();

            for (i, &nft_address) in listing.nfts.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                // #[cfg(not(test))]
                // {
                //     // Transfer NFT to buyer
                //     if self.supports_interface_check(nft_address, INTERFACE_ID_ERC721) {
                //         use erc721::Erc721;
                //         let erc721_instance: Erc721 =
                //             ink_env::call::FromAccountId::from_account_id(nft_address);
                //         ensure!(
                //             erc721_instance
                //                 .transfer_from(self.env().caller(), creator, token_id)
                //                 .is_ok(),
                //             Error::NotOwningItem
                //         );
                //     } else if self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155) {
                //         use erc1155::Erc1155;
                //         let erc1155_instance: Erc1155 =
                //             ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //         ensure!(
                //             erc1155_instance.transfer_from(
                //                 self.env().caller(),
                //                 creator,
                //                 token_id,
                //                 quantity,
                //                 Vec::new()
                //             ),
                //             Error::NotOwningItem
                //         );
                //     }
                //     use address_registry::AddressRegistry;
                //     let address_registry_instance: AddressRegistry =
                //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

                //     ensure!(
                //         AccountId::from([0x0; 32])
                //             == address_registry_instance.bundle_marketplace(),
                //         Error::InvalidPayToken
                //     );
                //     let marketplace_instance: BundleMarketplace =
                //         ink_env::call::FromAccountId::from_account_id(
                //             address_registry_instance.marketplace(),
                //         );
                //     marketplace_instance.validate_item_sold(nft_address, token_id, owner, creator);
                // }
            }
            self.listings.remove(&(self.env().caller(), _bundle_id.clone()));
            listing.price = 0;
            self.listings.insert(&(creator, _bundle_id.clone()), &listing);
            self.owners.insert(&_bundle_id, &creator);
            self.offers.remove(&(_bundle_id.clone(), creator));

            self.env().emit_event(ItemSold {
                seller: self.env().caller(),
                buyer: creator,
                bundle_id:bundle_id.clone(),
                pay_token: offer.pay_token,
                unit_price: self.get_price(offer.pay_token),
                price: offer.price,
            });
            self.env().emit_event(OfferCanceled { creator, bundle_id });
            Ok(())
        }

        fn get_price(&self, pay_token: AccountId) -> Balance {
            let mut unit_price = 0;
            // #[cfg(not(test))]
            // {
            //     ensure!(
            //         AccountId::from([0x0; 32]) != self.address_registry,
            //         Error::InvalidPayToken
            //     );
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     let marketplace_instance: PriceSeed = ink_env::call::FromAccountId::from_account_id(
            //         address_registry_instance.marketplace(),
            //     );

            //     unit_price = marketplace_instance.get_price(pay_token);
            // }
            unit_price
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn valid_owner(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            owner: AccountId,
            quantity: u128,
        ) -> Result<()> {
            // #[cfg(not(test))]
            // {
            //     if self.supports_interface_check(nft_address, INTERFACE_ID_ERC721) {
            //         use erc721::Erc721;
            //         let erc721_instance: Erc721 =
            //             ink_env::call::FromAccountId::from_account_id(nft_address);
            //         ensure!(
            //             owner == erc721_instance.owner_of(token_id),
            //             Error::NotOwningItem
            //         );
            //     } else if self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155) {
            //         use erc1155::Erc1155;
            //         let erc1155_instance: Erc1155 =
            //             ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //         ensure!(
            //             quantity <= erc1155_instance.balance_of(owner, token_id),
            //             Error::NotOwningItem
            //         );
            //     } else {
            //         ensure!(false, Error::InvalidNFTAddress);
            //     }
            // }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn valid_pay_token(&self, pay_token: AccountId) -> Result<()> {
            if AccountId::from([0x0; 32]) != pay_token {
                // #[cfg(not(test))]
                // {
                //     use address_registry::AddressRegistry;
                //     let address_registry_instance: AddressRegistry =
                //         ink_env::call::FromAccountId::from_account_id(self.address_registry);
                //     ensure!(
                //         AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
                //         Error::InvalidPayToken
                //     );
                //     let token_registry_instance: TokenRegistry =
                //         ink_env::call::FromAccountId::from_account_id(
                //             address_registry_instance.token_registry(),
                //         );
                //     ensure!(
                //         token_registry_instance.enabled(pay_token),
                //         Error::InvalidPayToken,
                //     );
                // }
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: Vec<u8>) -> bool {
            // This is disabled during tests due to the use of `invoke_contract()` not being
            // supported (tests end up panicking).
            let mut ans = false;
            // #[cfg(not(test))]
            // {
            //     use ink_env::call::{build_call, Call, ExecutionInput, Selector};
            //     let supports_interface_selector: [u8; 4] = [0xF2, 0x3A, 0x6E, 0x61];
            //     // If our recipient is a smart contract we need to see if they accept or
            //     // reject this transfer. If they reject it we need to revert the call.
            //     let params = build_call::<Environment>()
            //         .call_type(Call::new().callee(callee).gas_limit(5000))
            //         .exec_input(
            //             ExecutionInput::new(Selector::new(supports_interface_selector))
            //                 .push_arg(data),
            //         )
            //         .returns::<Vec<u8>>()
            //         .params();

            //     match ink_env::invoke_contract(&params) {
            //         Ok(v) => {
            //             ink_env::debug_println!(
            //                 "Received return value \"{:?}\" from contract {:?}",
            //                 v,
            //                 data
            //             );
            //             ans = v == &data[..];
            //             // assert_eq!(
            //             //     v,
            //             //     &ON_ERC_1155_RECEIVED_SELECTOR[..],
            //             //     "The recipient contract at {:?} does not accept token transfers.\n
            //             //     Expected: {:?}, Got {:?}",
            //             //     to,
            //             //     ON_ERC_1155_RECEIVED_SELECTOR,
            //             //     v
            //             // )
            //         }
            //         Err(e) => {
            //             match e {
            //                 ink_env::Error::CodeNotFound | ink_env::Error::NotCallable => {
            //                     // Our recipient wasn't a smart contract, so there's nothing more for
            //                     // us to do
            //                     ink_env::debug_println!(
            //                         "Recipient at {:?} from is not a smart contract ({:?})",
            //                        callee,
            //                         e
            //                     );
            //                 }
            //                 _ => {
            //                     // We got some sort of error from the call to our recipient smart
            //                     // contract, and as such we must revert this call
            //                     // panic!("Got error \"{:?}\" while trying to call {:?}", e, from)
            //                 }
            //             }
            //         }
            //     }
            // }
            ans
        }

        /**
        @notice Method for updating platform fee
        @dev Only admin
        @param _platformFee uint256 the platform fee to set
        */
        #[ink(message)]
        pub fn update_platform_fee(&mut self, platform_fee: Balance) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.platform_fee = platform_fee;
            self.env().emit_event(UpdatePlatformFee { platform_fee });
            Ok(())
        }
        /**
        @notice Method for updating platform fee address
        @dev Only admin
        @param fee_recipient payable address the address to sends the funds to
        */
        #[ink(message)]
        pub fn update_platform_fee_recipient(&mut self, fee_recipient: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.fee_recipient = fee_recipient;
            self.env()
                .emit_event(UpdatePlatformFeeRecipient { fee_recipient });
            Ok(())
        }
        /**
        @notice Update SubAddressRegistry contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_address_registry(&mut self, address_registry: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.address_registry = address_registry;

            Ok(())
        }

        /**
         * @notice Validate and cancel listing
         * @dev Only marketplace can access
         */
        #[ink(message)]
        pub fn validate_item_sold(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            quantity: u128,
        ) -> Result<()> {
            //onlyContract
            // #[cfg(not(test))]
            // {
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     ensure!(
            //         self.env().caller() == address_registry_instance.auction()
            //             || self.env().caller() == address_registry_instance.marketplace(),
            //         Error::SenderMustBeAuctionOrMarketplace
            //     );
            // }
            let items = self.bundle_ids_per_item.get(&(nft_address, token_id)).unwrap();
            for _bundle_id in &items {
                let owner = self.owners.get(&_bundle_id).unwrap();
                if owner != AccountId::from([0x0; 32]) {
                    let mut listing = self.listings.get(&(owner, _bundle_id.clone())).unwrap();
                    let bundle_id = self.bundle_ids.get(&_bundle_id).unwrap();
                    let index = self
                        .nft_indices
                        .get(&(_bundle_id.clone(), nft_address, token_id))
                        .unwrap() as usize;
                    if listing.quantities[index] > quantity {
                        listing.quantities[index] -= quantity;
                    } else {
                        self.nft_indices
                            .remove(&(_bundle_id.clone(), nft_address, token_id));
                        if listing.nfts.len() == 1 {
                            self.listings.remove(&(owner, _bundle_id.clone()));
                            self.owners.remove(&_bundle_id);
                            self.bundle_ids.remove(&_bundle_id);
                            self.env().emit_event(ItemUpdated {
                                owner: self.env().caller(),
                                bundle_id,
                                nft: Vec::new(),
                                token_id: Vec::new(),
                                quantity: Vec::new(),
                                pay_token: AccountId::from([0x0; 32]),
                                new_price: 0,
                            });
                            continue;
                        } else {
                            let indexu=index as u128;
                            if index < listing.nfts.len() - 1 {
                                let last_index=listing.nfts.len() - 1;
                                listing.nfts.swap(index, last_index);
                                let last_index=listing.token_ids.len() - 1;
                                listing.token_ids.swap(index, last_index);
                                let last_index=listing.quantities.len() - 1;
                                listing.quantities.swap(index, last_index);
                                self.nft_indices.insert(
                                    &(_bundle_id.clone(), listing.nfts[index], listing.token_ids[index]),
                                    &indexu,
                                );
                            }
                            listing.nfts.pop();
                            listing.token_ids.pop();
                            listing.quantities.pop();
                        }
                    }
                    self.env().emit_event(ItemUpdated {
                        owner: self.env().caller(),
                        bundle_id,
                        nft: listing.nfts,
                        token_id: listing.token_ids,
                        quantity: listing.quantities,
                        pay_token: listing.pay_token,
                        new_price: listing.price,
                    });
                }
            }

            self.bundle_ids_per_item.remove(&(nft_address, token_id));
            Ok(())
        }

        fn get_now(&self) -> u128 {
            self.env().block_timestamp().into()
        }
        fn get_bundle_id(&self, bundle_id: &String) -> String {
            use ink_env::hash;

            let uncompressed = bundle_id.as_bytes();

            // Hash the uncompressed public key by Keccak256 algorithm.
            let mut hash = <hash::Keccak256 as hash::HashOutput>::Type::default();
            // The first byte indicates that the public key is uncompressed.
            // Let's skip it for hashing the public key directly.
            ink_env::hash_bytes::<hash::Keccak256>(&uncompressed[1..], &mut hash);
            bundle_id.clone()
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
