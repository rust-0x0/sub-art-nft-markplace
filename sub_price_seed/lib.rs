//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_price_seed::{SubPriceSeed, SubPriceSeedRef};

use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
mod sub_price_seed {
    // use ink_lang as ink;
    // use ink_prelude::string::String;
    use ink_storage::{
        traits::{SpreadAllocate},
        Mapping,
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubPriceSeed {
        /// @notice keeps track of oracles for each tokens
        oracles: Mapping<AccountId, AccountId>,
        // address registry contract
        address_registry: AccountId,
        /// @notice wrapped SUB contract
        wsub: AccountId,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwner,
        OracleAlreadySet,
        InvalidPayToken,
        OracleNotSet,
        TransactionFailed,
    }
    pub type Result<T> = core::result::Result<T, Error>;
    impl SubPriceSeed {
        /// Creates a new token contract.
        #[ink(constructor)]
        pub fn new(address_registry: AccountId, wsub: AccountId) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.address_registry = address_registry;
                contract.wsub = wsub;
            })
        }

        /**
        @notice Register oracle contract to token
        @dev Only owner can register oracle
        @param _token ERC20 token address
        @param _oracle Oracle address
        */
        #[ink(message)]
        pub fn register_oracle(&mut self, token: AccountId, oracle: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.ensure_token_registry_enabled(token)?;
            ensure!(
                self.oracles
                    .get(&token)
                    .unwrap_or(AccountId::from([0x0; 32]))
                    == AccountId::from([0x0; 32]),
                Error::OracleAlreadySet
            );
            self.oracles.insert(&token, &oracle);
            Ok(())
        }
        fn ensure_token_registry_enabled(&self, token: AccountId) -> Result<()> {
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.token_registry(),
                    Error::InvalidPayToken
                );
                ensure!(
                    self.token_registry_enabled(address_registry_instance.token_registry(), token)
                        .is_ok(),
                    Error::InvalidPayToken,
                );
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn token_registry_enabled(&self, callee: AccountId, token: AccountId) -> Result<bool> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput};
                let selector: [u8; 4] = [0x14, 0x14, 0x63, 0x1C]; //0x1414631c enabled
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
        /**
        @notice Update oracle address for token
        @dev Only owner can update oracle
        @param _token ERC20 token address
        @param _oracle Oracle address
        */
        #[ink(message)]
        pub fn update_oracle(&mut self, token: AccountId, oracle: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                self.oracles
                    .get(&token)
                    .unwrap_or(AccountId::from([0x0; 32]))
                    != AccountId::from([0x0; 32]),
                Error::OracleNotSet
            );
            self.oracles.insert(&token, &oracle);
            Ok(())
        }

        #[ink(message)]
        pub fn get_price(&self, token: AccountId) -> (u128, u32) {
            if self
                .oracles
                .get(&token)
                .unwrap_or(AccountId::from([0x0; 32]))
                == AccountId::from([0x0; 32])
            {
                return (0, 0);
            }
            // IOracle oracle = IOracle(oracles[_token]);
            // return (oracle.latestAnswer(), oracle.decimals());
            (0, 0)
        }
        #[ink(message)]
        pub fn wsub(&self) -> AccountId {
            self.wsub
        }
        /**
        @notice Update address registry contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_address_registry(&mut self, address_registry: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.address_registry = address_registry;
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
