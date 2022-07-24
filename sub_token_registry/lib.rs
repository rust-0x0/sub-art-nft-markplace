//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_token_registry::{SubTokenRegistry,SubTokenRegistryRef};

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
    use ink_lang as ink;
    use ink_prelude::string::String;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
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
            ensure!(!self.enabled.get(&token).unwrap_or(false), Error::TokenAlreadyAdded);
            self.enabled.insert(&token,&true);
        self.env().emit_event(TokenAdded {
                token,
             });
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
            ensure!(self.enabled.get(&token).unwrap_or(false), Error::TokenNotExist);
            self.enabled.remove(&token);
        self.env().emit_event(TokenRemoved {
                token,
             });
            Ok(())
        }
          #[ink(message)]
        pub fn enabled(&self,token:AccountId)->bool{
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
    }
}
