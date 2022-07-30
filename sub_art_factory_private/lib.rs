//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_art_factory_private::{SubArtFactoryPrivate, SubArtFactoryPrivateRef};

#[cfg_attr(test, allow(dead_code))]
const INTERFACE_ID_ERC1155: [u8; 4] = [0xD9, 0xB6, 0x7A, 0x26];
use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
mod sub_art_factory_private {
    // use ink_lang as ink;
    use ink_lang::codegen::EmitEvent;
    use ink_prelude::string::String;
    // use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use scale::{Decode, Encode};
    use sub_art_tradable_private::sub_art_tradable_private::{ContractCreated, ContractDisabled};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubArtFactoryPrivate {
        /// @notice Marketplace contract
        marketplace: AccountId,
        /// @notice BundleMarketplace contract
        bundle_marketplace: AccountId,
        /// # note NFT mint fee
        mint_fee: Balance,
        /// # note Platform fee
        platform_fee: Balance,
        /// # note Platform fee receipient
        fee_recipient: AccountId,
        exists: Mapping<AccountId, bool>,
        code_hash: Hash,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwner,
        InsufficientFunds,
        TransferFailed,
        TransferOwnershipFailed,
        ArtContractAlreadyRegistered,
        NotAnERC1155Contract,
        ArtContractIsNotRegistered,
        TransactionFailed,
    }

    // The SubArtFactory result types.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SubArtFactoryPrivate {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(
            marketplace: AccountId,
            bundle_marketplace: AccountId,
            mint_fee: Balance,
            platform_fee: Balance,
            fee_recipient: AccountId,
            code_hash: Hash,
        ) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.marketplace = marketplace;
                contract.bundle_marketplace = bundle_marketplace;
                contract.mint_fee = mint_fee;
                contract.platform_fee = platform_fee;
                contract.fee_recipient = fee_recipient;
                contract.code_hash = code_hash;
            })
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
        #[ink(message)]
        pub fn update_mint_fee(&mut self, mint_fee: Balance) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.mint_fee = mint_fee;
            Ok(())
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
            Ok(())
        }

        /// @notice Method for deploy new SubArtTradable contract
        /// @param _name Name of NFT contract
        /// @param _symbol Symbol of NFT contract
        #[ink(message, payable)]
        #[cfg_attr(test, allow(unused_variables))]
        pub fn create_nft_contract(&mut self, name: String, symbol: String) -> Result<AccountId> {
            ensure!(
                self.env().transferred_value() >= self.platform_fee,
                Error::InsufficientFunds
            );
            ensure!(
                self.env()
                    .transfer(self.fee_recipient, self.env().transferred_value())
                    .is_ok(),
                Error::TransferFailed
            );
            let instantiate_contract = || -> Result<AccountId> {
                #[cfg(test)]
                {
                    ink_env::debug_println!("ans:{:?}", 1);
                    Ok(AccountId::from([0xAA; 32]))
                }
                #[cfg(not(test))]
                {
                    use sub_art_tradable_private::SubArtTradablePrivateRef;
                    let total_balance = Self::env().balance();
                    let version: u32 = 1;
                    let salt = version.to_le_bytes();
                    let instance_params = SubArtTradablePrivateRef::new(
                        name,
                        symbol,
                        self.marketplace,
                        self.bundle_marketplace,
                        self.mint_fee,
                        self.fee_recipient,
                    )
                    .endowment(total_balance / 4)
                    .code_hash(self.code_hash)
                    .salt_bytes(salt)
                    .params();
                    let init_result = ink_env::instantiate_contract(&instance_params);
                    let contract_addr =
                        init_result.expect("failed at instantiating the `Erc1155` contract");
                    let mut sub_art_tradable_private_instance: SubArtTradablePrivateRef =
                        ink_env::call::FromAccountId::from_account_id(contract_addr);
                    let _r =
                        sub_art_tradable_private_instance.transfer_ownership(self.env().caller());
                    ensure!(_r.is_ok(), Error::TransferOwnershipFailed);

                    Ok(contract_addr)
                }
            };
            let ans_contract_addr = instantiate_contract()?;
            self.exists.insert(&ans_contract_addr, &true);
            self.env().emit_event(ContractCreated {
                creator: self.env().caller(),
                nft_address: ans_contract_addr,
            });
            Ok(ans_contract_addr)
        }

        /// @notice Method for registering existing SubArtTradable contract
        /// @param  tokenContractAddress Address of NFT contract
        #[ink(message)]
        pub fn register_token_contract(&mut self, token_contract: AccountId) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                !self.exists.get(&token_contract).unwrap_or(false),
                Error::ArtContractAlreadyRegistered
            );
            ensure!(
                self.supports_interface_check(token_contract, crate::INTERFACE_ID_ERC1155),
                Error::NotAnERC1155Contract
            );

            self.exists.insert(&token_contract, &true);
            self.env().emit_event(ContractCreated {
                creator: self.env().caller(),
                nft_address: token_contract,
            });
            Ok(())
        }
        /// @notice Method for disabling existing SubArtTradable contract
        /// @param  tokenContractAddress Address of NFT contract
        #[ink(message)]
        pub fn disable_token_contract(&mut self, token_contract: AccountId) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);

            ensure!(
                self.exists.get(&token_contract).unwrap_or(false),
                Error::ArtContractIsNotRegistered
            );

            self.exists.insert(&token_contract, &false);
            self.env().emit_event(ContractDisabled {
                caller: self.env().caller(),
                nft_address: token_contract,
            });
            Ok(())
        }
        #[ink(message)]
        pub fn exists(&self, token: AccountId) -> bool {
            self.exists.get(&token).unwrap_or(false)
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: [u8; 4]) -> bool {
            #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}", 1);
                true
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
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;
        type Event = <sub_art_tradable_private::SubArtTradablePrivate as ::ink_lang::reflect::ContractEventBase>::Type;
        fn set_caller(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }
        fn default_accounts() -> ink_env::test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>()
        }
        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account_id, balance)
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
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
        fn django() -> AccountId {
            default_accounts().django
        }

        fn eve() -> AccountId {
            default_accounts().eve
        }

        fn frank() -> AccountId {
            default_accounts().frank
        }
        fn fee_recipient() -> AccountId {
            default_accounts().django
        }
        fn init_contract() -> SubArtFactoryPrivate {
            let erc = SubArtFactoryPrivate::new(
                frank(),
                eve(),
                1,
                1,
                fee_recipient(),
                Hash::from([0x99; 32]),
            );

            erc
        }
        #[ink::test]
        fn update_marketplace_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            let marketplace = bob();
            assert!(art_factory.update_marketplace(marketplace).is_ok());

            assert_eq!(art_factory.marketplace, marketplace);
        }

        #[ink::test]
        fn update_bundle_marketplace_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            let bundle_marketplace = bob();
            assert!(art_factory
                .update_bundle_marketplace(bundle_marketplace)
                .is_ok());

            assert_eq!(art_factory.bundle_marketplace, bundle_marketplace);
        }

        #[ink::test]
        fn update_mint_fee_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            let mint_fee = 10;
            assert!(art_factory.update_mint_fee(mint_fee).is_ok());

            assert_eq!(art_factory.mint_fee, mint_fee);
        }

        #[ink::test]
        fn update_platform_fee_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            let platform_fee = 10;
            assert!(art_factory.update_platform_fee(platform_fee).is_ok());

            assert_eq!(art_factory.platform_fee, platform_fee);
        }

        #[ink::test]
        fn update_platform_fee_recipient_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            let fee_recipient = bob();
            assert!(art_factory
                .update_platform_fee_recipient(fee_recipient)
                .is_ok());

            assert_eq!(art_factory.fee_recipient, fee_recipient);
        }
        #[ink::test]
        fn create_nft_contract_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            set_balance(caller, 10);
            set_balance(fee_recipient(), 0);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(1);

            let contract_addr =
                art_factory.create_nft_contract(String::from("test"), String::from("TEST"));
            // assert_eq!(contract_addr.unwrap_err(),Error::TransferOwnershipFailed);
            assert!(contract_addr.is_ok());

            // // Token 1 does not exists.
            assert_eq!(art_factory.exists.get(&contract_addr.unwrap()), Some(true));
            assert_eq!(get_balance(fee_recipient()), 1);
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_contract_created_event(&emitted_events[0], caller, contract_addr.unwrap());
        }

        #[ink::test]
        fn register_token_contract_works() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            set_balance(caller, 10);
            set_balance(fee_recipient(), 0);
            let token_contract = django();
            // assert_eq!(art_factory
            //     .register_token_contract(
            //       token_contract,
            //     ).unwrap_err(),Error::TransferOwnershipFailed);
            assert!(art_factory.register_token_contract(token_contract,).is_ok());

            // // Token 1 does not exists.
            assert_eq!(art_factory.exists.get(&token_contract), Some(true));
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_contract_created_event(&emitted_events[0], caller, token_contract);
        }

        #[ink::test]
        fn disable_token_contract() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            set_balance(caller, 10);
            set_balance(fee_recipient(), 0);
            let token_contract = django();
            art_factory.exists.insert(&token_contract, &true);
            // assert_eq!(contract_addr.unwrap_err(),Error::TransferOwnershipFailed);
            assert!(art_factory.disable_token_contract(token_contract,).is_ok());

            // // Token 1 does not exists.
            assert_eq!(art_factory.exists.get(&token_contract), Some(false));
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_contract_disabled_event(&emitted_events[0], caller, token_contract);
        }

        #[ink::test]
        fn exists_contract() {
            // Create a new contract instance.
            let mut art_factory = init_contract();
            let caller = alice();
            set_caller(caller);
            set_balance(caller, 10);
            set_balance(fee_recipient(), 0);
            let token_contract = charlie();
            art_factory.exists.insert(&token_contract, &true);

            // // Token 1 does not exists.
            assert!(art_factory.exists(token_contract));
        }

        fn assert_contract_created_event(
            event: &ink_env::test::EmittedEvent,
            expected_creator: AccountId,
            expected_nft_address: AccountId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ContractCreated(ContractCreated {
                creator,
                nft_address,
            }) = decoded_event
            {
                assert_eq!(
                    creator, expected_creator,
                    "encountered invalid ContractCreated.creator"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ContractCreated.nft_address"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ContractCreated event")
            }
        }

        fn assert_contract_disabled_event(
            event: &ink_env::test::EmittedEvent,
            expected_caller: AccountId,
            expected_nft_address: AccountId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::ContractDisabled(ContractDisabled {
                caller,
                nft_address,
            }) = decoded_event
            {
                assert_eq!(
                    caller, expected_caller,
                    "encountered invalid ContractDisabled.caller"
                );
                assert_eq!(
                    nft_address, expected_nft_address,
                    "encountered invalid ContractDisabled.nft_address"
                );
            } else {
                panic!("encountered unexpected event kind: expected a ContractDisabled event")
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
    }
}
