//! # ERC-721
//!
//! This is an ERC-721 Token implementation.

#![cfg_attr(not(feature = "std"), no_std)]

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
mod sub_artion {
    #[cfg_attr(test, allow(dead_code))]
    const INTERFACE_ID_ERC721: [u8; 4] = [0x80, 0xAC, 0x58, 0xCD];

    use ink_lang as ink;
    use ink_prelude::string::String;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u128;

    /// The interface for an ERC-1155 compliant contract.
    ///
    /// The interface is defined here: <https://eips.ethereum.org/EIPS/eip-1155>.
    ///
    /// The goal of ERC-1155 is to allow a single contract to manage a variety of assets.
    /// These assets can be fungible, non-fungible, or a combination.
    ///
    /// By tracking multiple assets the ERC-1155 standard is able to support batch transfers, which
    /// make it easy to transfer a mix of multiple tokens at once.
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
    pub struct SubArtion {
        /// Mapping from token to owner.
        token_owner: Mapping<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: Mapping<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: Mapping<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: Mapping<(AccountId, AccountId), ()>,
        ///  current max tokenId
        token_id_pointer: TokenId,
        ///  TokenID -> Uri
        token_uris: Mapping<TokenId, String>,
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
    }

    // The SubArtion result types.
    pub type Result<T> = core::result::Result<T, Error>;
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
    impl SubArtion {
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

        #[ink(message)]
        pub fn supports_interface(&self, interface_id: [u8; 4]) -> bool {
            INTERFACE_ID_ERC721 == interface_id
        }
        /// Creates a new token.
        #[ink(message, payable)]
        pub fn mint_with_owner_and_uri(
            &mut self,
            beneficiary: AccountId,
            token_uri: String,
        ) -> Result<TokenId> {
            ensure!(
                self.env().transferred_value() >= self.platform_fee,
                Error::InsufficientFundsToMint
            );
            let minter = self.env().caller();

            self._assert_minting_params_valid(&token_uri, minter)?;
            self.token_id_pointer += 1;
            let token_id = self.token_id_pointer;
            let caller = beneficiary;
            // Mint token and set token URI
            self.add_token_to(&caller, token_id)?;
            // _setTokenURI(tokenId, token_uri);  //TODO
            self.set_token_uri(token_id, &token_uri)?;
            // Send NativeToken fee to fee recipient
            ensure!(
                self.env()
                    .transfer(self.fee_recipient, self.env().transferred_value())
                    .is_ok(),
                Error::InsufficientFundsToMint
            );
            self.env().emit_event(Minted {
                token_id,
                beneficiary,
                token_uri,
                minter,
            });
            Ok(token_id)
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn_art(&mut self, token_id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            ensure!(
                self.owner_of(token_id) == Some(caller) || self.is_approved(token_id, caller),
                Error::OnlyOwnerOrApproved
            );

            self.burn(token_id)?;

            Ok(())
        }

        ///checks the given token ID is approved either for all or the single token ID
        fn is_approved(&self, token_id: TokenId, operator: AccountId) -> bool {
            self.is_approved_for_all(
                self.owner_of(token_id)
                    .unwrap_or(AccountId::from([0x0; 32])),
                operator,
            ) || self.get_approved(token_id) == Some(operator)
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

    impl Erc721 for SubArtion {
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
