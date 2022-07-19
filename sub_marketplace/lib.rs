//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]

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
    use ink_lang as ink;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };

    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u32;

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
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
    }

    // The SubMarketplace result types.
    pub type Result<T> = core::result::Result<T, Error>;
    /// Event emitted when a token ItemListed occurs.
    #[ink(event)]
    pub struct ItemListed {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
        quantity: u128,
        pay_token: AccountId,
        price_per_item: Balance,
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
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
        quantity: u128,
        pay_token: AccountId,
        unit_price: Balance,
        price_per_item: Balance,
    }

    /// Event emitted when a token ItemUpdated occurs.
    #[ink(event)]
    pub struct ItemUpdated {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
        pay_token: AccountId,
        new_price: Balance,
    }

    #[ink(event)]
    pub struct ItemCanceled {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
    }

    /// Event emitted when a token OfferCreated occurs.
    #[ink(event)]
    pub struct OfferCreated {
        #[ink(topic)]
        creator: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
        quantity: u128,
        pay_token: AccountId,
        price_per_item: Balance,
        deadline: u128,
    }
    /// Event emitted when a token OfferCanceled occurs.
    #[ink(event)]
    pub struct OfferCanceled {
        #[ink(topic)]
        creator: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        token_id: TokenId,
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
            //             quantity <= erc1155_instance.balance_of(self.env().caller(),token_id),
            //             Error::MustHoldEnoughNFTs
            //         );
            //         ensure!(
            //             erc1155_instance
            //                 .is_approved_for_all(self.env().caller(), self.env().account_id()),
            //             Error::ItemNotApproved
            //         );
            //     }
            // }
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
            // #[cfg(not(test))]
            // {
            //     use erc20::Erc20;
            //     let erc20_instance: Erc20 =
            //         ink_env::call::FromAccountId::from_account_id(auction.pay_token);
            //     let result = erc20_instance.transfer_from(
            //         self.env().caller(),
            //         self.fee_recipient,
            //         fee_amount,
            //     );
            //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
            // }
            let minter = self.minters.get(&(nft_address, token_id)).unwrap();
            let royalty = self.royalties.get(&(nft_address, token_id)).unwrap();
            if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                let royalty_fee = (price - fee_amount) * royalty / 10000;
                // #[cfg(not(test))]
                // {
                //     use erc20::Erc20;
                //     let erc20_instance: Erc20 =
                //         ink_env::call::FromAccountId::from_account_id(auction.pay_token);
                //     let result =
                //         erc20_instance.transfer_from(self.env().caller(), minter, royalty_fee);
                //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
                // }
                fee_amount += royalty_fee;
            } else {
                let collection_royalty = self.collection_royalties.get(nft_address).unwrap();
                let minter = collection_royalty.fee_recipient;
                let royalty = collection_royalty.royalty;
                if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                    let royalty_fee = (price - fee_amount) * royalty / 10000;
                    // #[cfg(not(test))]
                    // {
                    //     use erc20::Erc20;
                    //     let erc20_instance: Erc20 =
                    //         ink_env::call::FromAccountId::from_account_id(auction.pay_token);
                    //     let result =
                    //         erc20_instance.transfer_from(self.env().caller(), minter, royalty_fee);
                    //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
                    // }
                    fee_amount += royalty_fee;
                }
            }

            // #[cfg(not(test))]
            // {
            //     use erc20::Erc20;
            //     let erc20_instance: Erc20 =
            //         ink_env::call::FromAccountId::from_account_id(auction.pay_token);
            //     let result =
            //         erc20_instance.transfer_from(self.env().caller(), owner, price - fee_amount);
            //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);

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
            //         AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
            //         Error::InvalidPayToken
            //     );
            //     let bundle_marketplace_instance: BundleMarketplace =
            //         ink_env::call::FromAccountId::from_account_id(
            //             address_registry_instance.bundle_marketplace(),
            //         );
            //     bundle_marketplace_instance.validate_item_sold(
            //         nft_address,
            //         token_id,
            //         listing.quantity,
            //     );
            // }

            self.env().emit_event(ItemSold {
                seller: owner,
                buyer: self.env().caller(),
                nft_address,
                token_id,
                quantity: listing.quantity,
                pay_token,
                unit_price: self.get_price(pay_token),
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
            creator: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
            quantity: u128,
            pay_token: AccountId,
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
            // #[cfg(not(test))]
            // {
            //     use erc1155::Erc1155;
            //     let erc1155_instance: Erc1155 =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     ensure!(
            //         self.supports_interface_check(nft_address, INTERFACE_ID_ERC721)
            //             || self.supports_interface_check(nft_address, INTERFACE_ID_ERC1155),
            //         Error::InvalidNFTAddress
            //     );
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     ensure!(
            //         AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
            //         Error::InvalidPayToken
            //     );
            //     let auction_instance: SubAuction = ink_env::call::FromAccountId::from_account_id(
            //         address_registry_instance.auction(),
            //     );
            //     let (_, _, start_time, _, resulted) =
            //         auction_instance.get_auction(nft_address, token_id);
            //     ensure!(
            //         0 == start_time || resulted,
            //         Error::CannotPlaceAnOfferIfAuctionIsGoingOn
            //     );
            // }
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
                // #[cfg(not(test))]
                // {
                //     use erc20::Erc20;
                //     let erc20_instance: Erc20 =
                //         ink_env::call::FromAccountId::from_account_id(offer.pay_token);
                //     let result = erc20_instance.transfer_from(creator, minter, royalty_fee);
                //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
                // }
                fee_amount += royalty_fee;
            } else {
                let collection_royalty = self.collection_royalties.get(nft_address).unwrap();
                let minter = collection_royalty.fee_recipient;
                let royalty = collection_royalty.royalty;
                if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                    let royalty_fee = (price - fee_amount) * royalty / 10000;
                    // #[cfg(not(test))]
                    // {
                    //     use erc20::Erc20;
                    //     let erc20_instance: Erc20 =
                    //         ink_env::call::FromAccountId::from_account_id(offer.pay_token);
                    //     let result = erc20_instance.transfer_from(creator, minter, royalty_fee);
                    //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
                    // }
                    fee_amount += royalty_fee;
                }
            }

            // #[cfg(not(test))]
            // {
            //     use erc20::Erc20;
            //     let erc20_instance: Erc20 =
            //         ink_env::call::FromAccountId::from_account_id(offer.pay_token);
            //     let result =
            //         erc20_instance.transfer_from(creator, self.env().caller(), price - fee_amount);
            //     ensure!(result.is_ok(), Error::InsufficientBalanceOrNotApproved);
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
            //                 offer.quantity,
            //                 Vec::new()
            //             ),
            //             Error::NotOwningItem
            //         );
            //     }
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     ensure!(
            //         AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
            //         Error::InvalidPayToken
            //     );
            //     let bundle_marketplace_instance: BundleMarketplace =
            //         ink_env::call::FromAccountId::from_account_id(
            //             address_registry_instance.bundle_marketplace(),
            //         );
            //     bundle_marketplace_instance.validate_item_sold(
            //         nft_address,
            //         token_id,
            //         offer.quantity,
            //     );
            // }

            self.env().emit_event(ItemSold {
                seller: self.env().caller(),
                buyer: creator,
                nft_address,
                token_id,
                quantity: offer.quantity,
                pay_token: offer.pay_token,
                unit_price: self.get_price(offer.pay_token),
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
            let mut ans = true;
            // #[cfg(not(test))]
            // {
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     let factory_instance: NFTFactory = ink_env::call::FromAccountId::from_account_id(
            //         address_registry_instance.factory(),
            //     );
            //     let private_factory_instance: NFTFactoryPrivate =
            //         ink_env::call::FromAccountId::from_account_id(
            //             address_registry_instance.private_factory(),
            //         );
            //     let art_factory_instance: NFTFactory =
            //         ink_env::call::FromAccountId::from_account_id(
            //             address_registry_instance.art_factory(),
            //         );
            //     let private_art_factory_instance: NFTFactoryPrivate =
            //         ink_env::call::FromAccountId::from_account_id(
            //             address_registry_instance.private_art_factory(),
            //         );
            //     ans = nft_address == address_registry_instance.artion()
            //         || factory_instance.exists(nft_address)
            //         || private_factory_instance.exists(nft_address)
            //         || art_factory_instance.exists(nft_address)
            //         || private_art_factory_instance.exists(nft_address);
            // }
            ans
        }

        fn get_price(&self, pay_token: AccountId) -> Balance {
            let mut unit_price = 0;
            #[cfg(not(test))]
            {
                ensure!(
                    AccountId::from([0x0; 32]) != self.address_registry,
                    Error::InvalidPayToken
                );
                use address_registry::AddressRegistry;
                let address_registry_instance: AddressRegistry =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                let price_feed_instance: PriceSeed = ink_env::call::FromAccountId::from_account_id(
                    address_registry_instance.price_feed(),
                );
                let (_unit_price, _decimals) = if AccountId::from([0x0; 32]) == pay_token {
                    price_feed_instance.get_price(price_feed_instance.wsub())
                } else {
                    price_feed_instance.get_price(pay_token)
                };
                if _decimals < 18 {
                    _unit_price *= 10u128.pow(18 - _decimals);
                } else {
                    _unit_price /= 10u128.pow(_decimals - 18);
                }
                unit_price = _unit_price;
            }
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
            //                         from,
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
        @notice Update FantomAddressRegistry contract
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
            // #[cfg(not(test))]
            // {
            //     use address_registry::AddressRegistry;
            //     let address_registry_instance: AddressRegistry =
            //         ink_env::call::FromAccountId::from_account_id(self.address_registry);

            //     ensure!(
            //         AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
            //         Error::InvalidPayToken
            //     );
            //     ensure!(
            //         self.env().caller() == address_registry_instance.bundle_marketplace(),
            //         Error::SenderMustBeBundleMarketplace
            //     );
            // }
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

        fn get_now(&self) -> u128 {
            self.env().block_timestamp().into()
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
