//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_nft_tradable::{SubNFTTradable, SubNFTTradableRef};

use ink_lang as ink;

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
pub mod sub_nft_tradable {
    #[cfg_attr(test, allow(dead_code))]
    const INTERFACE_ID_ERC721: [u8; 4] = [0x80, 0xAC, 0x58, 0xCD];

    use ink_lang as ink;
    use ink_prelude::string::String;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u128;

    #[ink::trait_definition]
    pub trait Erc721 {
        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32;

        /// Returns the owner of the token.
        #[ink(message)]
        fn owner_of(&self, id: TokenId) -> Option<AccountId>;

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        fn get_approved(&self, id: TokenId) -> Option<AccountId>;

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool;

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<()>;

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        fn approve(&mut self, to: AccountId, id: TokenId) -> Result<()>;

        /// Transfer approved or owned token.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, id: TokenId) -> Result<()>;
        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        fn transfer(&mut self, destination: AccountId, id: TokenId) -> Result<()>;

        /// Creates a new token.
        #[ink(message)]
        fn mint(&mut self, id: TokenId) -> Result<()>;

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        fn burn(&mut self, id: TokenId) -> Result<()>;
    }
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubNFTTradable {
        /// Mapping from token to owner.
        token_owner: Mapping<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: Mapping<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: Mapping<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: Mapping<(AccountId, AccountId), ()>,
        ///  current max tokenId
        current_token_id: TokenId,
        ///  TokenID -> Uri
        token_uris: Mapping<TokenId, String>,
        name: String,
        symbol: String,
        /// @notice Auction contract
        auction: AccountId,
        /// @notice Marketplace contract
        marketplace: AccountId,
        /// @notice BundleMarketplace contract
        bundle_marketplace: AccountId,
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
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotFetchValue,
        NotAllowed,
        InsufficientFundsToMint,
        OnlyOwnerOrApproved,
        OnlyOwner,
        TokenURIIsEmpty,
        DesignerIsZeroAddress,
        TokenShouldExist,
        OperatorQueryForNonExistentToken,
        TransferFailed,
        NewOwnerIsTheZeroAddress,
    }

    // The SubNFTTradable result types.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct ContractCreated {
        pub creator: AccountId,
        pub nft: AccountId,
    }
    #[ink(event)]
    pub struct ContractDisabled {
        pub caller: AccountId,
        pub nft: AccountId,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }
    #[ink(event)]
    pub struct OwnershipTransferred {
        #[ink(topic)]
        previous_owner: AccountId,
        #[ink(topic)]
        new_owner: AccountId,
    }
    /// Event emitted when a token Minted occurs.
    #[ink(event)]
    pub struct Minted {
        token_id: TokenId,
        beneficiary: AccountId,
        token_uri: String,
        minter: AccountId,
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
    impl SubNFTTradable {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            auction: AccountId,
            marketplace: AccountId,
            bundle_marketplace: AccountId,
            platform_fee: Balance,
            fee_recipient: AccountId,
        ) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.name = name;
                contract.symbol = symbol;
                contract.auction = auction;
                contract.marketplace = marketplace;
                contract.bundle_marketplace = bundle_marketplace;
                contract.platform_fee = platform_fee;
                contract.fee_recipient = fee_recipient;
            })
        }
        #[ink(message)]
        pub fn supports_interface(&self, interface_id: [u8; 4]) -> bool {
            INTERFACE_ID_ERC721 == interface_id
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
         * @dev Mints a token to an address with a tokenURI.
         * @param _to address of the future owner of the token
         */
        #[ink(message, payable)]
        pub fn mint_to_and_uri(&mut self, to: AccountId, token_uri: String) -> Result<TokenId> {
            ensure!(
                self.env().transferred_value() >= self.platform_fee,
                Error::InsufficientFundsToMint
            );
            let minter = self.env().caller();

            self._assert_minting_params_valid(&token_uri, minter)?;
            let token_id = self.get_next_token_id();
            // Mint token and set token URI
            self.add_token_to(&to, token_id)?;
            // _setTokenURI(tokenId, token_uri);  //TODO
            self.set_token_uri(token_id, &token_uri)?;
            self.increment_token_type_id();
            // Send NativeToken fee to fee recipient
            ensure!(
                self.env()
                    .transfer(self.fee_recipient, self.env().transferred_value())
                    .is_ok(),
                Error::TransferFailed
            );
            self.env().emit_event(Minted {
                token_id,
                beneficiary: to,
                token_uri,
                minter,
            });
            Ok(token_id)
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn_nft(&mut self, token_id: TokenId) -> Result<()> {
            let operator = self.env().caller();
            ensure!(
                self.owner_of(token_id) == Some(operator) || self.is_approved(token_id, operator),
                Error::OnlyOwnerOrApproved
            );

            self.burn(token_id)?;

            Ok(())
        }
        /**
         * @dev Returns whether the specified token exists by checking to see if it has a creator
         * @param _id uint256 ID of the token to query the existence of
         * @return bool whether the token exists
         */
        // fn exists(&self, id: TokenId) -> bool {
        //     if let Some(addr) = self.creators.get(&id) {
        //         addr != AccountId::from([0x0; 32])
        //     } else {
        //         false
        //     }
        // }
        /**
         * @dev calculates the next token ID based on value of _currentTokenID
         * @return uint256 for the next token ID
         */
        fn get_next_token_id(&self) -> TokenId {
            self.current_token_id + 1
        }
        /**
         * @dev increments the value of _currentTokenID
         */
        fn increment_token_type_id(&mut self) {
            self.current_token_id += 1;
        }
        ///checks the given token ID is approved either for all or the single token ID
        fn is_approved(&self, token_id: TokenId, operator: AccountId) -> bool {
            self.is_approved_for_all(
                self.owner_of(token_id)
                    .unwrap_or(AccountId::from([0x0; 32])),
                operator,
            ) || self.get_approved(token_id) == Some(operator)
        }
        #[ink(message)]
        pub fn is_approved_for_all_nft(&self, owner: AccountId, operator: AccountId) -> bool {
            if self.auction == operator
                || self.marketplace == operator
                || self.bundle_marketplace == operator
            {
                return true;
            }
            self.is_approved_for_all(owner, operator)
        }
        pub fn is_approved_or_owner(
            &mut self,
            spender: AccountId,
            token_id: TokenId,
        ) -> Result<bool> {
            ensure!(
                self.exists(token_id),
                Error::OperatorQueryForNonExistentToken
            );
            let owner = self.owner_of(token_id).unwrap();
            if self.is_approved_for_all_nft(owner, spender) {
                return Ok(true);
            }

            Ok(self.approved_or_owner(Some(spender), token_id))
        }
        pub fn set_token_uri(&mut self, token_id: TokenId, uri: &String) -> Result<()> {
            ensure!(self.exists(token_id), Error::TokenShouldExist);
            self.token_uris.insert(&token_id, uri);

            Ok(())
        }
        /**
        @notice Checks that the URI is not empty and the designer is a real address
        @param token_uri URI supplied on minting
        @param designer Address supplied on minting
        */
        fn _assert_minting_params_valid(
            &self,
            token_uri: &String,
            designer: AccountId,
        ) -> Result<()> {
            ensure!(!token_uri.is_empty(), Error::TokenURIIsEmpty);
            ensure!(
                designer != AccountId::from([0x0; 32]),
                Error::DesignerIsZeroAddress
            );
            Ok(())
        }
        ///ERC721 ==============
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                AccountId::default() != new_owner,
                Error::NewOwnerIsTheZeroAddress
            );
            let previous_owner = self.owner;
            self.owner = new_owner;
            self.env().emit_event(OwnershipTransferred {
                previous_owner,
                new_owner,
            });
            Ok(())
        }
        /// Transfers token `id` `from` the sender to the `to` `AccountId`.
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: TokenId,
        ) -> Result<()> {
            let caller = self.env().caller();
            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };
            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            };
            self.clear_approval(id);
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(&mut self, from: &AccountId, id: TokenId) -> Result<()> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.get(&id).is_none() {
                return Err(Error::TokenNotFound);
            }

            let count = owned_tokens_count
                .get(&from)
                .map(|c| c - 1)
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(&from, &count);
            token_owner.remove(&id);

            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: TokenId) -> Result<()> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.get(&id).is_some() {
                return Err(Error::TokenExists);
            }

            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            let count = owned_tokens_count.get(to).map(|c| c + 1).unwrap_or(1);

            owned_tokens_count.insert(to, &count);
            token_owner.insert(&id, to);

            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(&mut self, to: AccountId, approved: bool) -> Result<()> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed);
            }
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });

            if approved {
                self.operator_approvals.insert((&caller, &to), &());
            } else {
                self.operator_approvals.remove((&caller, &to));
            }

            Ok(())
        }

        /// Approve the passed `AccountId` to transfer the specified token on behalf of the message's sender.
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.owner_of(id);
            if !(owner == Some(caller)
                || self.approved_for_all(owner.expect("Error with AccountId"), caller))
            {
                return Err(Error::NotAllowed);
            };

            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            if self.token_approvals.get(&id).is_some() {
                return Err(Error::CannotInsert);
            } else {
                self.token_approvals.insert(&id, to);
            }

            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id,
            });

            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) {
            self.token_approvals.remove(&id);
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u32 {
            self.owned_tokens_count.get(of).unwrap_or(0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.operator_approvals.get((&owner, &operator)).is_some()
        }

        /// Returns true if the `AccountId` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                    || from == self.token_approvals.get(&id)
                    || self.approved_for_all(
                        owner.expect("Error with AccountId"),
                        from.expect("Error with AccountId"),
                    ))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.token_owner.get(&id).is_some()
        }
    }

    impl Erc721 for SubNFTTradable {
        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        /// Returns the owner of the token.
        #[ink(message)]
        fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&id)
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        fn get_approved(&self, id: TokenId) -> Option<AccountId> {
            self.token_approvals.get(&id)
        }

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.approved_for_all(owner, operator)
        }

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<()> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        fn approve(&mut self, to: AccountId, id: TokenId) -> Result<()> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, id: TokenId) -> Result<()> {
            self.transfer_token_from(&from, &to, id)?;
            Ok(())
        }
        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        fn transfer(&mut self, destination: AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        fn mint(&mut self, id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            self.add_token_to(&caller, id)?;
            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(caller),
                id,
            });
            Ok(())
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        fn burn(&mut self, id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            let owner = token_owner.get(&id).ok_or(Error::TokenNotFound)?;
            if owner != caller {
                return Err(Error::NotOwner);
            };

            let count = owned_tokens_count
                .get(&caller)
                .map(|c| c - 1)
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(&caller, &count);
            token_owner.remove(&id);

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(AccountId::from([0x0; 32])),
                id,
            });

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
