//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_token_registry::{SubTokenRegistry, SubTokenRegistryRef};

use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
mod sub_token_registry {
    // use ink_lang as ink;
    // use ink_prelude::string::String;
    use ink_storage::{
        traits::{SpreadAllocate},
        Mapping,
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubTokenRegistry {
        /// ERC20 Address -> Bool
        enabled: Mapping<AccountId, bool>,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwner,
        TokenAlreadyAdded,
        TokenNotExist,
    }

    // The SubTokenRegistry result types.
    pub type Result<T> = core::result::Result<T, Error>;
    /// Event emitted when a token TokenAdded occurs.
    #[ink(event)]
    pub struct TokenAdded {
        token: AccountId,
    }
    /// Event emitted when a token TokenRemoved occurs.
    #[ink(event)]
    pub struct TokenRemoved {
        token: AccountId,
    }

    impl SubTokenRegistry {
        /// Creates a new token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
            })
        }

        /**
        @notice Method for adding payment token
        @dev Only admin
        @param token ERC20 token address
        */
        #[ink(message)]
        pub fn add(&mut self, token: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                !self.enabled.get(&token).unwrap_or(false),
                Error::TokenAlreadyAdded
            );
            self.enabled.insert(&token, &true);
            self.env().emit_event(TokenAdded { token });
            Ok(())
        }

        /**
        @notice Method for removing payment token
        @dev Only admin
        @param token ERC20 token address
        */
        #[ink(message)]
        pub fn remove(&mut self, token: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                self.enabled.get(&token).unwrap_or(false),
                Error::TokenNotExist
            );
            self.enabled.remove(&token);
            self.env().emit_event(TokenRemoved { token });
            Ok(())
        }
        #[ink(message)]
        pub fn enabled(&self, token: AccountId) -> bool {
            self.enabled.get(token).unwrap_or(false)
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

        fn init_contract() -> Contract {
            let mut erc = Contract::new();
            erc.balances.insert((alice(), 1), &10);
            erc.balances.insert((alice(), 2), &20);
            erc.balances.insert((bob(), 1), &10);

            erc
        }
        fn assert_token_added_event(
            event: &ink_env::test::EmittedEvent,
            expected_token: AccountId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::TokenAdded(TokenAdded { token }) = decoded_event {
                assert_eq!(
                    token, expected_token,
                    "encountered invalid TokenAdded.token"
                );
            } else {
                panic!("encountered unexpected event kind: expected a TokenAdded event")
            }
        }

        fn assert_token_removed_event(
            event: &ink_env::test::EmittedEvent,
            expected_token: AccountId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::TokenRemoved(TokenRemoved { token }) = decoded_event {
                assert_eq!(
                    token, expected_token,
                    "encountered invalid TokenRemoved.token"
                );
            } else {
                panic!("encountered unexpected event kind: expected a TokenRemoved event")
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
