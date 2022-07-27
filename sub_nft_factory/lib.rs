//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_nft_factory::{SubNFTFactory, SubNFTFactoryRef};

#[cfg_attr(test, allow(dead_code))]
const INTERFACE_ID_ERC721: [u8; 4] = [0x80, 0xAC, 0x58, 0xCD];
use ink_lang as ink;
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}
#[ink::contract]
pub mod sub_nft_factory {
    // use ink_lang as ink;
    use ink_lang::codegen::EmitEvent;
    use ink_prelude::string::String;
    // use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{ SpreadAllocate},
        Mapping,
    };
    use scale::{Decode, Encode};
    use sub_nft_tradable::sub_nft_tradable::{ContractCreated, ContractDisabled};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubNFTFactory {
        /// @notice Auction contract
        auction: AccountId,
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
        NFTContractAlreadyRegistered,
        NotAnERC721Contract,
        NFTContractIsNotRegistered,
TransactionFailed,
    }

    // The SubNFTFactory result types.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SubNFTFactory {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(
            auction: AccountId,
            marketplace: AccountId,
            bundle_marketplace: AccountId,
            mint_fee: Balance,
            fee_recipient: AccountId,
            platform_fee: Balance,
            code_hash: Hash,
        ) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.auction = auction;
                contract.marketplace = marketplace;
                contract.bundle_marketplace = bundle_marketplace;
                contract.mint_fee = mint_fee;
                contract.platform_fee = platform_fee;
                contract.fee_recipient = fee_recipient;
                contract.code_hash = code_hash;
            })
        }

        /**
        @notice Update Auction contract
        @dev Only admin
        */
        #[ink(message)]
        pub fn update_auction(&mut self, auction: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.auction = auction;
            Ok(())
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
        pub fn update_fee_recipient(&mut self, fee_recipient: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.fee_recipient = fee_recipient;
            Ok(())
        }

        /// @notice Method for deploy new SubNFTTradable contract
        /// @param _name Name of NFT contract
        /// @param _symbol Symbol of NFT contract
        #[ink(message, payable)]
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
            let instantiate_contract = ||->Result<AccountId>{ 
   #[cfg(test)]
            {
                ink_env::debug_println!("ans:{:?}",  1);
                Ok(AccountId::default())
            }
#[cfg(not(test))]
            {
                use sub_nft_tradable::SubNFTTradableRef;
                let total_balance = Self::env().balance();
                let version: u32 = 1;
                let salt = version.to_le_bytes();
                let instance_params = SubNFTTradableRef::new(
                    name,
                    symbol,
                    self.auction,
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
                    init_result.expect("failed at instantiating the `Erc721` contract");
                let mut sub_nft_tradable_instance: SubNFTTradableRef =
                    ink_env::call::FromAccountId::from_account_id(contract_addr);
                let _r = sub_nft_tradable_instance.transfer_ownership(self.env().caller());
                ensure!(_r.is_ok(), Error::TransferOwnershipFailed);

                 Ok(contract_addr)
            }};
            let ans_contract_addr = instantiate_contract()?;
           
            self.exists.insert(&ans_contract_addr, &true);
            self.env().emit_event(ContractCreated {
                creator: self.env().caller(),
                nft_address: ans_contract_addr,
            });
            Ok(ans_contract_addr)
        }

        /// @notice Method for registering existing SubNFTTradable contract
        /// @param  tokenContractAddress Address of NFT contract
        #[ink(message)]
        pub fn register_token_contract(&mut self, token_contract: AccountId) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            ensure!(
                !self.exists.get(&token_contract).unwrap_or(false),
                Error::NFTContractAlreadyRegistered
            );
            ensure!(
                self.supports_interface_check(token_contract, crate::INTERFACE_ID_ERC721),
                Error::NotAnERC721Contract
            );

            self.exists.insert(&token_contract, &true);
            self.env().emit_event(ContractCreated {
                creator: self.env().caller(),
                nft_address: token_contract,
            });
            Ok(())
        }
        /// @notice Method for disabling existing SubNFTTradable contract
        /// @param  tokenContractAddress Address of NFT contract
        #[ink(message)]
        pub fn disable_token_contract(&mut self, token_contract: AccountId) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);

            ensure!(
                self.exists.get(&token_contract).unwrap_or(false),
                Error::NFTContractIsNotRegistered
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
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        // use ink_lang as ink;
   use ink_env::Clear;
        type Event = <sub_nft_tradable::SubNFTTradable as ::ink_lang::reflect::ContractEventBase>::Type;
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

        fn init_contract() -> SubNFTFactory {
            let mut erc = SubNFTFactory::new(alice(),alice(),bob(),0,charlie(),0,Hash::from([0x99;32]));
       

            erc
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
