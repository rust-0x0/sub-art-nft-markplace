//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_marketplace::{SubMarketplace, SubMarketplaceRef};

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
mod sub_marketplace {
    // use ink_lang as ink;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };
    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u128;

    #[derive(Default, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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
        pub quantity: u128,
        pub pay_token: AccountId,
        pub price_per_item: Balance,
        pub starting_time: u128,
    }

    #[derive(Default, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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
        pub quantity: u128,
        pub price_per_item: Balance,
        pub deadline: u128,
    }

    #[derive(Default, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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
    pub struct CollectionRoyalty {
        pub royalty: u128,
        pub creator: AccountId,
        pub fee_recipient: AccountId,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubMarketplace {
        /// @notice NftAddress -> Token ID -> Minter
        minters: Mapping<(AccountId, TokenId), AccountId>,
        /// @notice NftAddress -> Token ID -> Royalty
        royalties: Mapping<(AccountId, TokenId), u128>,
        /// @notice NftAddress -> Token ID -> Owner -> Listing item
        listings: Mapping<(AccountId, TokenId, AccountId), Listing>,
        /// @notice NftAddress -> Token ID -> Offerer -> Offer
        offers: Mapping<(AccountId, TokenId, AccountId), Offer>,

        /// @notice NftAddress -> Royalty
        collection_royalties: Mapping<AccountId, CollectionRoyalty>,
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
        NotOwneAndOrContractNotApproved,
        TransactionFailed,
    }

    // The SubMarketplace result types.
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

    impl SubMarketplace {
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
        /// @notice Method for listing NFT
        /// @param _nftAddress Address of NFT contract
        /// @param _tokenId Token ID of NFT
        /// @param _quantity token amount to list (needed for ERC-1155 NFTs, set as 1 for ERC-721)
        /// @param _payToken Paying token
        /// @param _pricePerItem sale price for each iteam
        /// @param _startingTime scheduling for a future sale
        #[ink(message)]
        #[cfg_attr(test, allow(unused_variables))]
        pub fn list_item(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            quantity: u128,
            pay_token: AccountId,
            price_per_item: Balance,
            starting_time: u128,
        ) -> Result<()> {
            let listing = self
                .listings
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(listing.quantity == 0, Error::AlreadyListed);

            if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                ensure!(
                    Some(self.env().caller()) == self.erc721_owner_of(nft_address, token_id)?,
                    Error::NotOwningItem
                );

                ensure!(
                    self.erc721_is_approved_for_all(
                        nft_address,
                        self.env().caller(),
                        self.env().account_id(),
                    )
                    .unwrap_or(false),
                    Error::ItemNotApproved
                );
            } else if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155) {
                ensure!(
                    quantity <= self.erc1155_balance_of(nft_address, self.env().caller())?,
                    Error::MustHoldEnoughNFTs
                );
                ensure!(
                    self.erc1155_is_approved_for_all(
                        nft_address,
                        self.env().caller(),
                        self.env().account_id(),
                    )
                    .is_ok(),
                    Error::ItemNotApproved
                );
            }
            self.valid_pay_token(pay_token)?;
            self.listings.insert(
                (nft_address, token_id, self.env().caller()),
                &Listing {
                    quantity,
                    pay_token,
                    price_per_item,
                    starting_time,
                },
            );
            self.env().emit_event(ItemListed {
                owner: self.env().caller(),
                nft_address,
                token_id,
                quantity,
                pay_token,
                price_per_item,
                starting_time,
            });
            Ok(())
        }

        /// @notice Method for canceling listed NFT
        #[ink(message)]
        pub fn cancel_listing(&mut self, nft_address: AccountId, token_id: TokenId) -> Result<()> {
            let listing = self
                .listings
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(listing.quantity > 0, Error::NotListedItem);
            self._cancel_listing(nft_address, token_id, self.env().caller())?;
            Ok(())
        }
        fn _cancel_listing(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            owner: AccountId,
        ) -> Result<()> {
            let listing = self.listings.get(&(nft_address, token_id, owner)).unwrap();
            self.valid_owner(nft_address, token_id, owner, listing.quantity)?;
            self.listings.remove(&(nft_address, token_id, owner));
            self.env().emit_event(ItemCanceled {
                owner: self.env().caller(),
                nft_address,
                token_id,
            });
            Ok(())
        }

        /// @notice Method for updating listed NFT
        /// @param _nftAddress Address of NFT contract
        /// @param _tokenId Token ID of NFT
        /// @param _payToken payment token
        /// @param _newPrice New sale price for each iteam
        #[ink(message)]
        pub fn update_listing(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            new_price: Balance,
        ) -> Result<()> {
            let mut listing = self
                .listings
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(listing.quantity > 0, Error::NotListedItem);
            self.valid_owner(nft_address, token_id, self.env().caller(), listing.quantity)?;
            self.valid_pay_token(pay_token)?;
            listing.pay_token = pay_token;
            listing.price_per_item = new_price;
            self.env().emit_event(ItemUpdated {
                owner: self.env().caller(),
                nft_address,
                token_id,
                pay_token,
                new_price,
            });
            Ok(())
        }

        /// @notice Method for buying listed NFT
        /// @param _nftAddress NFT contract address
        /// @param _tokenId TokenId
        #[ink(message)]
        pub fn buy_item(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            owner: AccountId,
        ) -> Result<()> {
            let listing = self
                .listings
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(listing.quantity > 0, Error::NotListedItem);
            self.valid_owner(nft_address, token_id, owner, listing.quantity)?;

            ensure!(
                self.get_now() >= listing.starting_time,
                Error::ItemNotBuyable
            );

            ensure!(listing.pay_token == pay_token, Error::InvalidPayToken);

            self._buy_item(nft_address, token_id, pay_token, owner)?;
            Ok(())
        }
        fn _buy_item(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            owner: AccountId,
        ) -> Result<()> {
            let listing = self.listings.get(&(nft_address, token_id, owner)).unwrap();
            let price = listing.price_per_item * listing.quantity;
            let mut fee_amount = price * self.platform_fee / 1000;
            self.erc20_transfer_from(
                pay_token,
                self.env().caller(),
                self.fee_recipient,
                fee_amount,
            )?;
            let minter = self.minters.get(&(nft_address, token_id)).unwrap();
            let royalty = self.royalties.get(&(nft_address, token_id)).unwrap();
            if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                let royalty_fee = (price - fee_amount) * royalty / 10000;
                self.erc20_transfer_from(pay_token, self.env().caller(), minter, royalty_fee)?;
                fee_amount += royalty_fee;
            } else {
                let collection_royalty = self.collection_royalties.get(nft_address).unwrap();
                let minter = collection_royalty.fee_recipient;
                let royalty = collection_royalty.royalty;
                if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                    let royalty_fee = (price - fee_amount) * royalty / 10000;
                    self.erc20_transfer_from(pay_token, self.env().caller(), minter, royalty_fee)?;
                    fee_amount += royalty_fee;
                }
            }

            self.erc20_transfer_from(pay_token, self.env().caller(), owner, price - fee_amount)?;

            if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                self.erc721_transfer_from(nft_address, owner, self.env().caller(), token_id)?;
            } else if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155) {
                self.erc1155_transfer_from(
                    nft_address,
                    owner,
                    self.env().caller(),
                    token_id,
                    listing.quantity,
                )?;
            }
            self.bundle_marketplace_validate_item_sold(nft_address, token_id, listing.quantity)?;

            self.env().emit_event(ItemSold {
                seller: owner,
                buyer: self.env().caller(),
                nft_address,
                token_id,
                quantity: listing.quantity,
                pay_token,
                unit_price: self.get_price(pay_token)?,
                price_per_item: price / listing.quantity,
            });
            self.listings.remove(&(nft_address, token_id, owner));
            Ok(())
        }
        /// @notice Method for offering item
        /// @param _nftAddress NFT contract address
        /// @param _tokenId TokenId
        /// @param _payToken Paying token
        /// @param _quantity Quantity of items
        /// @param _pricePerItem Price per item
        /// @param _deadline Offer expiration
        #[ink(message)]
        pub fn create_offer(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            quantity: u128,
            price_per_item: Balance,
            deadline: u128,
        ) -> Result<()> {
            let offer = self
                .offers
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(
                offer.quantity == 0 || offer.deadline <= self.get_now(),
                Error::OfferAlreadyCreated
            );

            ensure!(
                self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721)
                    || self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155),
                Error::InvalidNFTAddress
            );

            let (start_time, resulted) = self.auction_start_time_resulted(nft_address, token_id)?;
            ensure!(
                0 == start_time || resulted,
                Error::CannotPlaceAnOfferIfAuctionIsGoingOn
            );

            ensure!(deadline > self.get_now(), Error::InvalidExpiration);
            self.valid_pay_token(pay_token)?;
            self.offers.insert(
                &(nft_address, token_id, self.env().caller()),
                &Offer {
                    quantity,
                    pay_token,
                    price_per_item,
                    deadline,
                },
            );
            self.env().emit_event(OfferCreated {
                creator: self.env().caller(),
                nft_address,
                token_id,
                quantity,
                pay_token,
                price_per_item,
                deadline,
            });
            Ok(())
        }
        /// @notice Method for canceling the offer
        /// @param _nftAddress NFT contract address
        /// @param _tokenId TokenId
        #[ink(message)]
        pub fn cancel_offer(&mut self, nft_address: AccountId, token_id: TokenId) -> Result<()> {
            let offer = self
                .offers
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            ensure!(
                offer.quantity > 0 || offer.deadline > self.get_now(),
                Error::OfferNotExistsOrExpired
            );
            self.offers
                .remove(&(nft_address, token_id, self.env().caller()));
            self.env().emit_event(OfferCanceled {
                creator: self.env().caller(),
                nft_address,
                token_id,
            });
            Ok(())
        }

        /// @notice Method for accepting the offer
        /// @param _nftAddress NFT contract address
        /// @param _tokenId TokenId
        /// @param _creator Offer creator address
        #[ink(message)]
        pub fn accept_offer(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            creator: AccountId,
        ) -> Result<()> {
            let offer = self.offers.get(&(nft_address, token_id, creator)).unwrap();
            ensure!(
                offer.quantity > 0 || offer.deadline > self.get_now(),
                Error::OfferNotExistsOrExpired
            );
            self.valid_owner(nft_address, token_id, self.env().caller(), offer.quantity)?;
            let price = offer.price_per_item * offer.quantity;
            let mut fee_amount = price * self.platform_fee / 1000;
            let minter = self.minters.get(&(nft_address, token_id)).unwrap();
            let royalty = self.royalties.get(&(nft_address, token_id)).unwrap();
            if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                let royalty_fee = (price - fee_amount) * royalty / 10000;
                self.erc20_transfer_from(offer.pay_token, creator, minter, royalty_fee)?;
                fee_amount += royalty_fee;
            } else {
                let collection_royalty = self.collection_royalties.get(nft_address).unwrap();
                let minter = collection_royalty.fee_recipient;
                let royalty = collection_royalty.royalty;
                if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                    let royalty_fee = (price - fee_amount) * royalty / 10000;
                    self.erc20_transfer_from(offer.pay_token, creator, minter, royalty_fee)?;
                    fee_amount += royalty_fee;
                }
            }
            self.erc20_transfer_from(
                offer.pay_token,
                creator,
                self.env().caller(),
                price - fee_amount,
            )?;

            // Transfer NFT to buyer
            if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                self.erc721_transfer_from(nft_address, self.env().caller(), creator, token_id)?;
            } else if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155) {
                self.erc1155_transfer_from(
                    nft_address,
                    self.env().caller(),
                    creator,
                    token_id,
                    offer.quantity,
                )?;
            }
            self.bundle_marketplace_validate_item_sold(nft_address, token_id, offer.quantity)?;

            self.env().emit_event(ItemSold {
                seller: self.env().caller(),
                buyer: creator,
                nft_address,
                token_id,
                quantity: offer.quantity,
                pay_token: offer.pay_token,
                unit_price: self.get_price(offer.pay_token)?,
                price_per_item: offer.price_per_item,
            });
            self.env().emit_event(OfferCanceled {
                creator,
                nft_address,
                token_id,
            });
            self.listings
                .remove(&(nft_address, token_id, self.env().caller()));
            self.offers.remove(&(nft_address, token_id, creator));
            Ok(())
        }
        /// @notice Method for setting royalty
        /// @param _nftAddress NFT contract address
        /// @param _tokenId TokenId
        /// @param _royalty Royalty
        #[ink(message)]
        pub fn register_royalty(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            royalty: u128,
        ) -> Result<()> {
            ensure!(royalty <= 10000, Error::InvalidRoyalty);
            ensure!(self.is_nft(nft_address), Error::InvalidNFTAddress);
            self.valid_owner(nft_address, token_id, self.env().caller(), 1)?;

            ensure!(
                self.minters.get(&(nft_address, token_id)).is_none(),
                Error::RoyaltyAlreadySet
            );
            self.minters
                .insert(&(nft_address, token_id), &self.env().caller());
            self.royalties.insert(&(nft_address, token_id), &royalty);
            Ok(())
        }
        /// @notice Method for setting royalty
        /// @param _nftAddress NFT contract address
        /// @param _royalty Royalty
        #[ink(message)]
        pub fn register_collection_royalty(
            &mut self,
            nft_address: AccountId,
            creator: AccountId,
            royalty: u128,
            fee_recipient: AccountId,
        ) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                AccountId::from([0x0; 32]) != creator,
                Error::InvalidCreatorAddress
            );
            ensure!(royalty <= 10000, Error::InvalidRoyalty);
            ensure!(
                royalty == 0 || AccountId::from([0x0; 32]) != fee_recipient,
                Error::InvalidCreatorAddress
            );
            ensure!(self.is_nft(nft_address), Error::InvalidNFTAddress);
            self.collection_royalties.insert(
                &nft_address,
                &CollectionRoyalty {
                    royalty,
                    creator,
                    fee_recipient,
                },
            );

            Ok(())
        }
        fn is_nft(&self, nft_address: AccountId) -> bool {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                nft_address == address_registry_instance.artion()
                    || self
                        .factory_exists(address_registry_instance.nft_factory(), nft_address)
                        .unwrap_or(false)
                    || self
                        .factory_exists(
                            address_registry_instance.nft_factory_private(),
                            nft_address,
                        )
                        .unwrap_or(false)
                    || self
                        .factory_exists(address_registry_instance.art_factory(), nft_address)
                        .unwrap_or(false)
                    || self
                        .factory_exists(
                            address_registry_instance.art_factory_private(),
                            nft_address,
                        )
                        .unwrap_or(false)
            }
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn factory_exists(&self, callee: AccountId, token: AccountId) -> Result<bool> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0xCA, 0x94, 0x23, 0x1F]; //factory_exists
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(callee)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(token))
                    .returns::<bool>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        #[ink(message)]
        pub fn get_price(&self, pay_token: AccountId) -> Result<Balance> {
            #[cfg(not(test))]
            {
                ensure!(
                    AccountId::from([0x0; 32]) != self.address_registry,
                    Error::InvalidPayToken
                );
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                let price_seed_instance: sub_price_seed::SubPriceSeedRef =
                    ink_env::call::FromAccountId::from_account_id(
                        address_registry_instance.price_seed(),
                    );
                let (mut unit_price, decimals) = if AccountId::from([0x0; 32]) == pay_token {
                    price_seed_instance.get_price(price_seed_instance.wsub())
                } else {
                    price_seed_instance.get_price(pay_token)
                };
                if decimals < 18 {
                    unit_price *= 10u128.pow(18 - decimals);
                } else {
                    unit_price /= 10u128.pow(decimals - 18);
                }
                Ok(unit_price) 
            }
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn valid_owner(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            owner: AccountId,
            quantity: u128,
        ) -> Result<()> {
            if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                ensure!(
                    Some(self.env().caller()) == self.erc721_owner_of(nft_address, token_id)?,
                    Error::NotOwningItem
                );
            } else if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155) {
                ensure!(
                    quantity <= self.erc1155_balance_of(nft_address, owner)?,
                    Error::NotOwningItem
                );
            } else {
                ensure!(false, Error::InvalidNFTAddress);
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn valid_pay_token(&self, pay_token: AccountId) -> Result<()> {
            if AccountId::from([0x0; 32]) != pay_token {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                ensure!(
                    AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
                    Error::InvalidPayToken
                );
                ensure!(
                    self.token_registry_enabled(self.address_registry_token_registry()?, pay_token)
                        .is_ok(),
                    Error::InvalidPayToken,
                );
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn address_registry_token_registry(&self) -> Result<AccountId> {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                ensure!(
                    AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
                    Error::InvalidPayToken
                );
                Ok(address_registry_instance.token_registry())
            }
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn token_registry_enabled(&self, callee: AccountId, token: AccountId) -> Result<bool> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x14, 0x14, 0x63, 0x1C]; //token_registry_enabled
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(callee)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(token))
                    .returns::<bool>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: [u8; 4]) -> bool {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0xE6, 0x11, 0x3A, 0x8A]; //supports_interface_check
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(callee)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(data))
                    .returns::<bool>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result.unwrap_or(false)
            }
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
         * @dev Only bundle marketplace can access
         */
        #[ink(message)]
        pub fn validate_item_sold(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            seller: AccountId,
            buyer: AccountId,
        ) -> Result<()> {
            //onlyMarketplace
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
                    Error::InvalidPayToken
                );
                ensure!(
                    self.env().caller() == address_registry_instance.bundle_marketplace(),
                    Error::SenderMustBeBundleMarketplace
                );
            }
            let listing = self
                .listings
                .get(&(nft_address, token_id, self.env().caller()))
                .unwrap();
            if listing.quantity > 0 {
                self._cancel_listing(nft_address, token_id, seller)?;
            }
            self.offers.remove(&(nft_address, token_id, buyer));
            self.env().emit_event(OfferCanceled {
                creator: self.env().caller(),
                nft_address,
                token_id,
            });
            Ok(())
        }
        #[ink(message)]
        pub fn minter_of(&self, owner: AccountId, token_id: TokenId) -> AccountId {
            self.minter_of_impl(owner, token_id)
        }
        #[ink(message)]
        pub fn royalty_of(&self, nft_address: AccountId, token_id: TokenId) -> u128 {
            self.royalty_of_impl(nft_address, token_id)
        }
        #[ink(message)]
        pub fn collection_royalty_of(&self, nft_address: AccountId) -> (AccountId, Balance) {
            let collection_royalty = self.collection_royalty_impl(nft_address);
            (collection_royalty.fee_recipient, collection_royalty.royalty)
        }
    }
    #[ink(impl)]
    impl SubMarketplace {
        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn minter_of_impl(&self, owner: AccountId, token_id: TokenId) -> AccountId {
            self.minters.get(&(owner, token_id)).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn royalty_of_impl(&self, nft_address: AccountId, token_id: TokenId) -> u128 {
            self.royalties
                .get(&(nft_address, token_id))
                .unwrap_or_default()
        }
        #[inline]
        fn listing_impl(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            owner: AccountId,
        ) -> Listing {
            self.listings
                .get(&(nft_address, token_id, owner))
                .unwrap_or_default()
        }
        #[inline]
        fn offer_impl(&self, nft_address: AccountId, token_id: TokenId, owner: AccountId) -> Offer {
            self.offers
                .get(&(nft_address, token_id, owner))
                .unwrap_or_default()
        }
        #[inline]
        fn collection_royalty_impl(&self, nft_address: AccountId) -> CollectionRoyalty {
            self.collection_royalties
                .get(&nft_address)
                .unwrap_or_default()
        }

        fn get_now(&self) -> u128 {
            self.env().block_timestamp().into()
        }

        fn erc721_transfer_from(
            &mut self,
            token: AccountId,
            from: AccountId,
            to: AccountId,
            token_id: TokenId,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x0B, 0x39, 0x6F, 0x18]; //erc721_transfer_from
                let (gas_limit, transferred_value) = (0, 0);
                let _ = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(from)
                            .push_arg(to)
                            .push_arg(token_id),
                    )
                    .returns::<()>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
            }
            Ok(())
        }
        fn erc1155_transfer_from(
            &mut self,
            token: AccountId,
            from: AccountId,
            to: AccountId,
            token_id: TokenId,
            value: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x53, 0x24, 0xD5, 0x56]; //erc1155 safe_transfer_from
                let (gas_limit, transferred_value) = (0, 0);
                let _ = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(from)
                            .push_arg(to)
                            .push_arg(token_id)
                            .push_arg(value)
                            .push_arg(Vec::<u8>::new()),
                    )
                    .returns::<()>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
            }
            Ok(())
        }
        fn erc20_transfer_from(
            &mut self,
            token: AccountId,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x0B, 0x39, 0x6F, 0x18]; //erc20 transfer_from
                let (gas_limit, transferred_value) = (0, 0);
                let _ = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(from)
                            .push_arg(to)
                            .push_arg(value),
                    )
                    .returns::<()>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
            }
            Ok(())
        }

        fn erc721_is_approved_for_all(
            &self,
            token: AccountId,
            owner: AccountId,
            operator: AccountId,
        ) -> Result<bool> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x0F, 0x59, 0x22, 0xE9]; //erc721 is_approved_for_all
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(owner)
                            .push_arg(operator),
                    )
                    .returns::<bool>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        fn erc721_owner_of(
            &self,
            token: AccountId,
            token_id: TokenId,
        ) -> Result<Option<AccountId>> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x99, 0x72, 0x0C, 0x1E]; //erc721 owner_of
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(token_id))
                    .returns::<Option<AccountId>>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        fn erc1155_is_approved_for_all(
            &self,
            token: AccountId,
            owner: AccountId,
            operator: AccountId,
        ) -> Result<bool> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x36, 0x03, 0x4D, 0x3E]; //erc1155 is_approved_for_all
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(owner)
                            .push_arg(operator),
                    )
                    .returns::<bool>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        fn erc1155_balance_of(&self, token: AccountId, owner: AccountId) -> Result<Balance> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x16, 0x4B, 0x9B, 0xA0]; //erc1155 balance_of
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(owner))
                    .returns::<Balance>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        fn auction_start_time_resulted(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<(u128, bool)> {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x39, 0xF0, 0xAB, 0x3E]; //auction get_auction_start_time_resulted
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(address_registry_instance.auction())
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(nft_address)
                            .push_arg(token_id),
                    )
                    .returns::<(u128, bool)>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
        }

        fn bundle_marketplace_validate_item_sold(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            quantity: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
                    Error::InvalidPayToken
                );
                self._bundle_marketplace_validate_item_sold(
                    address_registry_instance.bundle_marketplace(),
                    nft_address,
                    token_id,
                    quantity,
                )?;
            }
            Ok(())
        }
        fn _bundle_marketplace_validate_item_sold(
            &self,
            token: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
            quantity: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x5E, 0x38, 0x31, 0x94]; //_bundle_marketplace_validate_item_sold
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(
                        ExecutionInput::new(selector.into())
                            .push_arg(nft_address)
                            .push_arg(token_id)
                            .push_arg(quantity),
                    )
                    .returns::<(AccountId, Balance)>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ensure!(result.is_ok(), Error::TransactionFailed);
            }
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
        fn default_accounts() -> ink_env::test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn charlie() -> AccountId {
            default_accounts().charlie
        }

        fn init_contract() -> SubMarketplace {
            let mut erc = SubMarketplace::new();
           

            erc
        }
        fn assert_item_listed_event(
            event: &ink_env::test::EmittedEvent,
            expected_owner: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
            expected_quantity: u128,
            expected_pay_token: AccountId,
            expected_price_per_item: Balance,
            expected_starting_time: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemListed(ItemListed {
                owner,
                nft_address,
                token_id,
                quantity,
                pay_token,
                price_per_item,
                starting_time,
            }) = decoded_event
            {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemListed.owner"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ItemListed.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid ItemListed.token_id"
                );
                assert_eq!(
                    quantity, expected_quantity,
                    "encountered invalid ItemListed.quantity"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemListed.pay_token"
                );
                assert_eq!(
                    price_per_item, expected_price_per_item,
                    "encountered invalid ItemListed.price_per_item"
                );

                assert_eq!(
                    starting_time, expected_starting_time,
                    "encountered invalid ItemListed.starting_time"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ItemListed event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::ItemListed",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemListed::owner",
                    value: &expected_owner,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemListed::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }

        fn assert_item_sold_event(
            event: &ink_env::test::EmittedEvent,
            expected_seller: AccountId,
            expected_buyer: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
            expected_quantity: u128,
            expected_pay_token: AccountId,
            expected_unit_price: Balance,
            expected_price_per_item: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemSold(ItemSold {
                seller,
                buyer: creator,
                nft_address,
                token_id,
                quantity,
                pay_token,
                unit_price,
                price_per_item,
            }) = decoded_event
            {
                assert_eq!(
                    seller, expected_seller,
                    "encountered invalid ItemSold.seller"
                );
                assert_eq!(buyer, expected_buyer, "encountered invalid ItemSold.buyer");
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ItemSold.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid ItemSold.token_id"
                );
                assert_eq!(
                    quantity, expected_quantity,
                    "encountered invalid ItemSold.quantity"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemSold.pay_token"
                );
                assert_eq!(
                    unit_price, expected_unit_price,
                    "encountered invalid ItemSold.unit_price"
                );

                assert_eq!(
                    price_per_item, expected_price_per_item,
                    "encountered invalid ItemSold.price_per_item"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ItemSold event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::ItemSold",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemSold::seller",
                    value: &expected_seller,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemSold::buyer",
                    value: &expected_buyer,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemSold::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }

        fn assert_item_updated_event(
            event: &ink_env::test::EmittedEvent,
            expected_owner: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
            expected_pay_token: AccountId,
            expected_new_price: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemUpdated(ItemUpdated {
                owner,
                nft_address,
                token_id,
                pay_token,
                new_price,
            }) = decoded_event
            {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemUpdated.owner"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ItemUpdated.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid ItemUpdated.token_id"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemUpdated.pay_token"
                );
                assert_eq!(
                    price_per_item, expected_price_per_item,
                    "encountered invalid ItemUpdated.price_per_item"
                );

                assert_eq!(
                    new_price, expected_new_price,
                    "encountered invalid ItemUpdated.new_price"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ItemUpdated event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::ItemUpdated",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemUpdated::owner",
                    value: &expected_owner,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemUpdated::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }

        fn assert_item_canceled_event(
            event: &ink_env::test::EmittedEvent,
            expected_owner: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemCanceled(ItemCanceled {
                owner,
                nft_address,
                token_id,
            }) = decoded_event
            {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemCanceled.owner"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ItemCanceled.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid ItemCanceled.token_id"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ItemCanceled event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::ItemCanceled",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemCanceled::owner",
                    value: &expected_owner,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::ItemCanceled::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }

        fn assert_offer_created_event(
            event: &ink_env::test::EmittedEvent,
            expected_creator: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
            expected_quantity: u128,
            expected_pay_token: AccountId,
            expected_price_per_item: Balance,
            expected_deadline: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::OfferCreated(OfferCreated {
                creator,
                nft_address,
                token_id,
                quantity,
                pay_token,
                price_per_item,
                deadline,
            }) = decoded_event
            {
                assert_eq!(
                    creator, expected_creator,
                    "encountered invalid OfferCreated.creator"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid OfferCreated.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid OfferCreated.token_id"
                );
                assert_eq!(
                    quantity, expected_quantity,
                    "encountered invalid OfferCreated.quantity"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid OfferCreated.pay_token"
                );
                assert_eq!(
                    price_per_item, expected_price_per_item,
                    "encountered invalid OfferCreated.price_per_item"
                );

                assert_eq!(
                    deadline, expected_deadline,
                    "encountered invalid OfferCreated.deadline"
                );
            } else {
                panic!("encountered unexpected event kind: expected a OfferCreated event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::OfferCreated",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::OfferCreated::creator",
                    value: &expected_creator,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::OfferCreated::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }

        fn assert_offer_canceled_event(
            event: &ink_env::test::EmittedEvent,
            expected_creator: AccountId,
            expected_nft_address: AccountId,
            expected_token_id: TokenId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::OfferCanceled(OfferCanceled {
                creator,
                nft_address,
                token_id,
            }) = decoded_event
            {
                assert_eq!(
                    creator, expected_creator,
                    "encountered invalid OfferCanceled.creator"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid OfferCanceled.nft_address"
                );
                assert_eq!(
                    token_id, expected_token_id,
                    "encountered invalid OfferCanceled.token_id"
                );
            } else {
                panic!("encountered unexpected event kind: expected a OfferCanceled event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Contract::OfferCanceled",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::OfferCanceled::creator",
                    value: &expected_creator,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Contract::OfferCanceled::nft_address",
                    value: &expected_nft_address,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::clear();
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {}",
                    n
                );
            }
        }
        fn assert_platform_fee_event(
            event: &ink_env::test::EmittedEvent,
            expected_platform_fee: bool,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::UpdatePlatformFee(UpdatePlatformFee { platform_fee }) = decoded_event {
                assert_eq!(
                    platform_fee, expected_platform_fee,
                    "encountered invalid UpdatePlatformFee.platform_fee"
                );
            } else {
                panic!("encountered unexpected event kind: expected a UpdatePlatformFee event")
            }
        }

        fn assert_platform_fee_recipient_event(
            event: &ink_env::test::EmittedEvent,
            expected_fee_recipient: bool,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::UpdatePlatformFeeRecipient(UpdatePlatformFeeRecipient { fee_recipient }) =
                decoded_event
            {
                assert_eq!(
                    fee_recipient, expected_fee_recipient,
                    "encountered invalid UpdatePlatformFeeRecipient.fee_recipient"
                );
            } else {
                panic!("encountered unexpected event kind: expected a UpdatePlatformFeeRecipient event")
            }
        }
        /// For calculating the event topic hash.
        struct PrefixedValue<'a, 'b, T> {
            pub prefix: &'a [u8],
            pub value: &'b T,
        }

        impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
        {
            #[inline]
            fn size_hint(&self) -> usize {
                self.prefix.size_hint() + self.value.size_hint()
            }

            #[inline]
            fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
                self.prefix.encode_to(dest);
                self.value.encode_to(dest);
            }
        }

        fn encoded_into_hash<T>(entity: &T) -> Hash
        where
            T: scale::Encode,
        {
            use ink_env::{
                hash::{Blake2x256, CryptoHash, HashOutput},
                Clear,
            };
            let mut result = Hash::clear();
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result;
            }
            let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }
    }
}
