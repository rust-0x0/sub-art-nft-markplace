//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

#![cfg_attr(not(feature = "std"), no_std)]
pub use self::sub_art_factory::{SubArtFactory,SubArtFactoryRef};


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
mod sub_art_factory {
    use ink_lang as ink;
 use ink_prelude::vec::Vec;
    use ink_prelude::string::String;
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };
    use sub_art_tradable::sub_art_tradable::{ContractCreated,ContractDisabled};
 use ink_lang::codegen::EmitEvent;
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubArtFactory {
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
        code_hash:Hash,
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
    }

 
    // The SubArtFactory result types.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SubArtFactory {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(
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
                contract.marketplace = marketplace;
                contract.bundle_marketplace= bundle_marketplace;
                contract.mint_fee = mint_fee;
                contract.fee_recipient = fee_recipient;
                contract.platform_fee = platform_fee;
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
        pub fn update_fee_recipient(&mut self, fee_recipient: AccountId) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.fee_recipient = fee_recipient;
            Ok(())
        }

        /// @notice Method for deploy new SubArtTradable contract
        /// @param _name Name of NFT contract
        /// @param _symbol Symbol of NFT contract
        #[ink(message, payable)]
        pub fn create_nft_contract(&mut self, name:String,symbol :String) -> Result<AccountId> {
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
            let mut ans_contract_addr = AccountId::from([0x0; 32]);
            #[cfg(not(test))]
            {
                use sub_art_tradable::SubArtTradableRef;
                let total_balance = Self::env().balance();
                let version:u32=1;
                let salt = version.to_le_bytes();
                let instance_params = SubArtTradableRef::new(name,symbol,self.marketplace,self.bundle_marketplace,self.mint_fee,self.fee_recipient)
                    .endowment(total_balance / 4)
                    .code_hash(self.code_hash)
                    .salt_bytes(salt)
                    .params();
                let init_result = ink_env::instantiate_contract(&instance_params);
                let contract_addr =
                    init_result.expect("failed at instantiating the `Erc1155` contract");
                let mut sub_art_tradable_instance: SubArtTradableRef =
                    ink_env::call::FromAccountId::from_account_id(contract_addr);
                let _r = sub_art_tradable_instance.transfer_ownership(self.env().caller());
                ensure!(_r.is_ok(), Error::TransferOwnershipFailed);

                ans_contract_addr = contract_addr;
            }
            self.exists.insert(&ans_contract_addr, &true);
            self.env().emit_event(ContractCreated {
                creator: self.env().caller(),
                nft: ans_contract_addr,
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
                nft: token_contract,
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
                nft: token_contract,
            });
            Ok(())
        }
   #[ink(message)]
        pub fn exists(&self,  token: AccountId) -> bool {
            self.exists.get(&token).unwrap_or(false)
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn supports_interface_check(&self, callee: AccountId, data: [u8;4]) -> bool {
            // This is disabled during tests due to the use of `invoke_contract()` not being
            // supported (tests end up panicking).
            let mut ans = false;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let supports_interface_selector: [u8; 4] = [0xF2, 0x3A, 0x6E, 0x61];
                // If our recipient is a smart contract we need to see if they accept or
                // reject this transfer. If they reject it we need to revert the call.
                let params = build_call::<Environment>()
                    .call_type(Call::new().callee(callee).gas_limit(5000))
                    .exec_input(
                        ExecutionInput::new(Selector::new(supports_interface_selector))
                            .push_arg(data),
                    )
                    .returns::<Vec<u8>>()
                    .params();

                match ink_env::invoke_contract(&params) {
                    Ok(v) => {
                        ink_env::debug_println!(
                            "Received return value \"{:?}\" from contract {:?}",
                            v,
                            data
                        );
                        ans = v == &data[..];
                        // assert_eq!(
                        //     v,
                        //     &ON_ERC_1155_RECEIVED_SELECTOR[..],
                        //     "The recipient contract at {:?} does not accept token transfers.\n
                        //     Expected: {:?}, Got {:?}",
                        //     to,
                        //     ON_ERC_1155_RECEIVED_SELECTOR,
                        //     v
                        // )
                    }
                    Err(e) => {
                        match e {
                            ink_env::Error::CodeNotFound | ink_env::Error::NotCallable => {
                                // Our recipient wasn't a smart contract, so there's nothing more for
                                // us to do
                                ink_env::debug_println!(
                                    "Recipient at {:?} from is not a smart contract ({:?})",
                                    callee,
                                    e
                                );
                            }
                            _ => {
                                // We got some sort of error from the call to our recipient smart
                                // contract, and as such we must revert this call
                                // panic!("Got error \"{:?}\" while trying to call {:?}", e, from)
                            }
                        }
                    }
                }
            }
            ans
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
