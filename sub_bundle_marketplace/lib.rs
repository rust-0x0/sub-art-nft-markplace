//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_bundle_marketplace::{SubBundleMarketplace, SubBundleMarketplaceRef};

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
    // use ink_lang as ink; 
    use ink_prelude::collections::BTreeSet;
    use ink_prelude::string::String;
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
        nft_addresses: Vec<AccountId>,
        token_ids: Vec<TokenId>,
        pub quantities: Vec<u128>,
        pub pay_token: AccountId,
        pub price: Balance,
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
        TransactionFailed,
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
        nft_addresses: Vec<AccountId>,
        token_ids: Vec<TokenId>,
        quantities: Vec<u128>,
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
            let _bundle_id = self.get_bundle_id(&bundle_id);
            let listing = self.listings.get(&(owner, _bundle_id)).unwrap();
            (
                listing.nft_addresses,
                listing.token_ids,
                listing.quantities,
                listing.price,
                listing.starting_time,
            )
        }

        #[cfg_attr(test, allow(unused_variables))]
        fn token_registry_enabled(&self, callee: AccountId, token: AccountId) -> Result<bool> {
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(false)
            }
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x14, 0x14, 0x63, 0x1C]; // token_registry_enabled
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
            bundle_id: String,
            nft_addresses: Vec<AccountId>,
            token_ids: Vec<TokenId>,
            quantities: Vec<u128>,
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

            #[cfg(not(test))]
            {
                ensure!(
                    AccountId::from([0x0; 32]) != self.address_registry,
                    Error::InvalidPayToken
                );
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                if pay_token != AccountId::from([0x0; 32]) {
                    ensure!(
                        AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
                        Error::InvalidPayToken
                    );
                    ensure!(
                        self.token_registry_enabled(
                            address_registry_instance.token_registry(),
                            pay_token
                        )
                        .unwrap_or(false),
                        Error::InvalidPayToken,
                    );
                }
            }
            listing.nft_addresses.clear();
            listing.token_ids.clear();
            listing.quantities.clear();
            for (i, &nft_address) in nft_addresses.iter().enumerate() {
                let token_id = token_ids[i];
                let quantity = quantities[i];

                #[cfg(not(test))]
                {
                    if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                        ensure!(
                            Some(self.env().caller())
                                == self.erc721_owner_of(nft_address, token_id)?,
                            Error::NotOwningItem
                        );
                        ensure!(
                            self.erc721_is_approved_for_all(
                                nft_address,
                                self.env().caller(),
                                self.env().account_id()
                            )
                            .unwrap_or(false),
                            Error::ItemNotApproved
                        );
                        listing.quantities.push(1);
                    } else if self
                        .supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155)
                    {
                        ensure!(
                            quantity
                                <= self.erc1155_balance_of(nft_address, self.env().caller())?,
                            Error::MustHoldEnoughNFTs
                        );
                        ensure!(
                            self.erc1155_is_approved_for_all(
                                nft_address,
                                self.env().caller(),
                                self.env().account_id()
                            )
                            .is_ok(),
                            Error::ItemNotApproved
                        );
                    } else {
                        ensure!(false, Error::InvalidNFTAddress);
                    }
                }
                listing.nft_addresses.push(nft_address);
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
            self.listings
                .insert(&(self.env().caller(), _bundle_id.clone()), &listing);
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
            for (i, &nft_address) in listing.nft_addresses.iter().enumerate() {
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
                nft_addresses: listing.nft_addresses,
                token_ids: listing.token_ids,
                quantities: listing.quantities,
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

            for (i, &nft_address) in listing.nft_addresses.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                #[cfg(not(test))]
                {
                    if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                        ensure!(
                            Some(self.env().caller())
                                == self.erc721_owner_of(nft_address, token_id)?,
                            Error::NotOwningItem
                        );
                        ensure!(
                            self.erc721_is_approved_for_all(
                                nft_address,
                                self.env().caller(),
                                self.env().account_id()
                            )
                            .unwrap_or(false),
                            Error::ItemNotApproved
                        );
                    } else if self
                        .supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155)
                    {
                        ensure!(
                            quantity <= self.erc1155_balance_of(nft_address, owner)?,
                            Error::MustHoldEnoughNFTs
                        );
                    } else {
                        ensure!(false, Error::InvalidNFTAddress);
                    }
                }
            }

            ensure!(
                self.get_now() >= listing.starting_time,
                Error::ItemNotBuyable
            );

            let price = listing.price;
            let fee_amount = price * self.platform_fee / 1000;
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
                ensure!(
                    self.erc20_transfer_from(
                        pay_token,
                        self.env().caller(),
                        self.fee_recipient,
                        fee_amount,
                    )
                    .is_ok(),
                    Error::FailedToSendTheOwnerFeeTransferFailed
                );
                ensure!(
                    self.erc20_transfer_from(
                        pay_token,
                        self.env().caller(),
                        owner,
                        price - fee_amount
                    )
                    .is_ok(),
                    Error::FailedToSendTheOwnerFeeTransferFailed
                );
            }

            for (i, &nft_address) in listing.nft_addresses.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                #[cfg(not(test))]
                {
                    if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                        self.erc721_transfer_from(
                            nft_address,
                            owner,
                            self.env().caller(),
                            token_id,
                        )?;
                    } else if self
                        .supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155)
                    {
                        self.erc1155_transfer_from(
                            nft_address,
                            owner,
                            self.env().caller(),
                            token_id,
                            quantity,
                        )?;
                    }
                    self.marketplace_validate_item_sold(
                        nft_address,
                        token_id,
                        owner,
                        self.env().caller(),
                    )?;
                }
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
                bundle_id: bundle_id.clone(),
                pay_token,
                unit_price: self.get_price(pay_token)?,
                price,
            });
            self.env().emit_event(OfferCanceled {
                creator: self.env().caller(),
                bundle_id,
            });
            Ok(())
        }
        fn marketplace_validate_item_sold(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            seller: AccountId,
            buyer: AccountId,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.marketplace(),
                    Error::InvalidPayToken
                );
                self._marketplace_validate_item_sold(
                    address_registry_instance.marketplace(),
                    nft_address,
                    token_id,
                    seller,
                    buyer,
                )?;
            }
            Ok(())
        }
        fn _marketplace_validate_item_sold(
            &self,
            token: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
            seller: AccountId,
            buyer: AccountId,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x5E, 0x38, 0x31, 0x94]; //_marketplace_validate_item_sold
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
                            .push_arg(seller)
                            .push_arg(buyer),
                    )
                    .returns::<(AccountId, Balance)>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ensure!(result.is_ok(), Error::TransactionFailed);
            }
            Ok(())
        }
        fn erc1155_is_approved_for_all(
            &self,
            token: AccountId,
            owner: AccountId,
            operator: AccountId,
        ) -> Result<bool> {
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(false)
            }
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

        fn marketplace_get_price(
            &self,
            token: AccountId,
            nft_address: AccountId,
        ) -> Result<Balance> {
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(Balance::default())
            }
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x99, 0x72, 0x0C, 0x1E]; //marketplace_get_price
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(nft_address))
                    .returns::<Balance>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                 result
            }
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
            let offer = self
                .offers
                .get(&(_bundle_id.clone(), self.env().caller()))
                .unwrap();
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

            let offer = self
                .offers
                .get(&(_bundle_id.clone(), self.env().caller()))
                .unwrap();
            ensure!(
                offer.deadline > self.get_now(),
                Error::OfferNotExistsOrExpired
            );
            self.offers
                .remove(&(_bundle_id.clone(), self.env().caller()));
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
            let fee_amount = price * self.platform_fee / 1000;
            self.erc20_transfer_from(offer.pay_token, creator, self.fee_recipient, fee_amount)?;
            self.erc20_transfer_from(
                offer.pay_token,
                self.env().caller(),
                owner,
                price - fee_amount,
            )?;

            let mut listing = self
                .listings
                .get(&(self.env().caller(), _bundle_id.clone()))
                .unwrap();

            for (i, &nft_address) in listing.nft_addresses.iter().enumerate() {
                let token_id = listing.token_ids[i];
                let quantity = listing.quantities[i];

                #[cfg(not(test))]
                {
                    // Transfer NFT to buyer
                    if self.supports_interface_check(nft_address, crate::INTERFACE_ID_ERC721) {
                        self.erc721_transfer_from(
                            nft_address,
                            self.env().caller(),
                            creator,
                            token_id,
                        )?;
                    } else if self
                        .supports_interface_check(nft_address, crate::INTERFACE_ID_ERC1155)
                    {
                        self.erc1155_transfer_from(
                            nft_address,
                            self.env().caller(),
                            creator,
                            token_id,
                            quantity,
                        )?;
                    }
                    self.marketplace_validate_item_sold(nft_address, token_id, owner, creator)?;
                }
            }
            self.listings
                .remove(&(self.env().caller(), _bundle_id.clone()));
            listing.price = 0;
            self.listings
                .insert(&(creator, _bundle_id.clone()), &listing);
            self.owners.insert(&_bundle_id, &creator);
            self.offers.remove(&(_bundle_id.clone(), creator));

            self.env().emit_event(ItemSold {
                seller: self.env().caller(),
                buyer: creator,
                bundle_id: bundle_id.clone(),
                pay_token: offer.pay_token,
                unit_price: self.get_price(offer.pay_token)?,
                price: offer.price,
            });
            self.env().emit_event(OfferCanceled { creator, bundle_id });
            Ok(())
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
                let selector: [u8; 4] = [0x0B, 0x39, 0x6F, 0x18]; //erc721 transfer_from
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
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(false)
            }
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
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(Some(AccountId::default()))
            }
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

        fn get_price(&self, pay_token: AccountId) -> Result<Balance> {
     #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(Balance::default())
            }
            #[cfg(not(test))]
            {
                ensure!(
                    AccountId::from([0x0; 32]) != self.address_registry,
                    Error::InvalidPayToken
                );
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                    self.marketplace_get_price(address_registry_instance.marketplace(), pay_token)
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
            #[cfg(not(test))]
            {
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
            }
            Ok(())
        }
        fn erc1155_balance_of(&self, token: AccountId, owner: AccountId) -> Result<Balance> {
            #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(Balance::default())
            }
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
        #[cfg_attr(test, allow(unused_variables))]
        fn valid_pay_token(&self, pay_token: AccountId) -> Result<()> {
            if AccountId::from([0x0; 32]) != pay_token {
                #[cfg(not(test))]
                {
                    let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                        ink_env::call::FromAccountId::from_account_id(self.address_registry);
                    ensure!(
                        AccountId::from([0x0; 32]) != address_registry_instance.token_registry(),
                        Error::InvalidPayToken
                    );
                    ensure!(
                        self.token_registry_enabled(
                            address_registry_instance.token_registry(),
                            pay_token
                        )
                        .unwrap_or(false),
                        Error::InvalidPayToken,
                    );
                }
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: [u8; 4]) -> bool {
            #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                false
            }
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
            // #[cfg(test)]
            // {
            //     ink_env::debug_println!("ans:{:?}",  ans);
            // }
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
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                ensure!(
                    self.env().caller() == address_registry_instance.auction()
                        || self.env().caller() == address_registry_instance.marketplace(),
                    Error::SenderMustBeBundleMarketplace
                );
            }
            let items = self
                .bundle_ids_per_item
                .get(&(nft_address, token_id))
                .unwrap();
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
                        if listing.nft_addresses.len() == 1 {
                            self.listings.remove(&(owner, _bundle_id.clone()));
                            self.owners.remove(&_bundle_id);
                            self.bundle_ids.remove(&_bundle_id);
                            self.env().emit_event(ItemUpdated {
                                owner: self.env().caller(),
                                bundle_id,
                                nft_addresses: Vec::new(),
                                token_ids: Vec::new(),
                                quantities: Vec::new(),
                                pay_token: AccountId::from([0x0; 32]),
                                new_price: 0,
                            });
                            continue;
                        } else {
                            let indexu = index as u128;
                            if index < listing.nft_addresses.len() - 1 {
                                let last_index = listing.nft_addresses.len() - 1;
                                listing.nft_addresses.swap(index, last_index);
                                let last_index = listing.token_ids.len() - 1;
                                listing.token_ids.swap(index, last_index);
                                let last_index = listing.quantities.len() - 1;
                                listing.quantities.swap(index, last_index);
                                self.nft_indices.insert(
                                    &(
                                        _bundle_id.clone(),
                                        listing.nft_addresses[index],
                                        listing.token_ids[index],
                                    ),
                                    &indexu,
                                );
                            }
                            listing.nft_addresses.pop();
                            listing.token_ids.pop();
                            listing.quantities.pop();
                        }
                    }
                    self.env().emit_event(ItemUpdated {
                        owner: self.env().caller(),
                        bundle_id,
                        nft_addresses: listing.nft_addresses,
                        token_ids: listing.token_ids,
                        quantities: listing.quantities,
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
   use ink_env::Clear;
        type Event = <SubBundleMarketplace as ::ink_lang::reflect::ContractEventBase>::Type;
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

        fn init_contract() -> SubBundleMarketplace {
            let mut erc = SubBundleMarketplace::new(alice(),0);
           

            erc
        }
        fn assert_item_listed_event(
            event: &ink_env::test::EmittedEvent,
            expected_owner: AccountId,
            expected_bundle_id: String,
            expected_pay_token: AccountId,
            expected_price: Balance,
            expected_starting_time: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemListed(ItemListed {
                owner,
                bundle_id,
                pay_token,
                price,
                starting_time,
            }) = decoded_event
            {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemListed.owner"
                );
                assert_eq!(
                    bundle_id, expected_bundle_id,
                    "encountered invalid ItemListed.bundle_id"
                );

                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemListed.pay_token"
                );
                assert_eq!(
                    price, expected_price,
                    "encountered invalid ItemListed.price"
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
            expected_bundle_id: String,
            expected_pay_token: AccountId,
            expected_unit_price: Balance,
            expected_price: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemSold(ItemSold {
                seller,
                buyer,
                bundle_id,
                pay_token,
                unit_price,
                price,
            }) = decoded_event
            {
                assert_eq!(
                    seller, expected_seller,
                    "encountered invalid ItemSold.seller"
                );
                assert_eq!(buyer, expected_buyer, "encountered invalid ItemSold.buyer");
                assert_eq!(
                    bundle_id, expected_bundle_id,
                    "encountered invalid ItemSold.bundle_id"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemSold.pay_token"
                );
                assert_eq!(
                    unit_price, expected_unit_price,
                    "encountered invalid ItemSold.unit_price"
                );

                assert_eq!(price, expected_price, "encountered invalid ItemSold.price");
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
            expected_bundle_id: String,
            expected_nft_addresses: Vec<AccountId>,
            expected_token_ids: Vec<TokenId>,
            expected_quantities: Vec<u128>,
            expected_pay_token: AccountId,
            expected_new_price: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemUpdated(ItemUpdated {
                owner,
                bundle_id,
                nft_addresses,
                token_ids,
                quantities,
                pay_token,
                new_price,
            }) = decoded_event
            {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemUpdated.owner"
                );
                assert_eq!(
                    nft_addresses, expected_nft_addresses,
                    "encountered invalid ItemUpdated.nft_addresses"
                );
                assert_eq!(
                    token_ids, expected_token_ids,
                    "encountered invalid ItemUpdated.token_ids"
                );
                assert_eq!(
                    quantities, expected_quantities,
                    "encountered invalid ItemUpdated.quantities"
                );
                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid ItemUpdated.pay_token"
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
            expected_bundle_id: String,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ItemCanceled(ItemCanceled { owner, bundle_id }) = decoded_event {
                assert_eq!(
                    owner, expected_owner,
                    "encountered invalid ItemCanceled.owner"
                );
                assert_eq!(
                    bundle_id, expected_bundle_id,
                    "encountered invalid ItemCanceled.bundle_id"
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
            expected_bundle_id: String,
            expected_pay_token: AccountId,
            expected_price: Balance,
            expected_deadline: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::OfferCreated(OfferCreated {
                creator,
                bundle_id,
                pay_token,
                price,
                deadline,
            }) = decoded_event
            {
                assert_eq!(
                    creator, expected_creator,
                    "encountered invalid OfferCreated.creator"
                );
                assert_eq!(
                    bundle_id, expected_bundle_id,
                    "encountered invalid OfferCreated.bundle_id"
                );

                assert_eq!(
                    pay_token, expected_pay_token,
                    "encountered invalid OfferCreated.pay_token"
                );
                assert_eq!(
                    price, expected_price,
                    "encountered invalid OfferCreated.price"
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
            expected_bundle_id: String,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::OfferCanceled(OfferCanceled { creator, bundle_id }) = decoded_event {
                assert_eq!(
                    creator, expected_creator,
                    "encountered invalid OfferCanceled.creator"
                );
                assert_eq!(
                    bundle_id, expected_bundle_id,
                    "encountered invalid OfferCanceled.bundle_id"
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
              expected_platform_fee: Balance,
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
             expected_fee_recipient: AccountId,
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
