//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!

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
pub mod sub_auction {
    use ink_lang as ink;
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
    pub struct Auction {
        /// The `AccountId` of the contract that is called in this transaction.
        pub owner: AccountId,
        /// The selector bytes that identifies the function of the callee that should be called.
        pub pay_token: AccountId,
        pub min_bid: Balance,
        /// The amount of chain balance that is transferred to the callee.
        pub reserve_price: Balance,
        /// Gas limit for the execution of the call.
        pub start_time: u128,
        /// The SCALE encoded parameters that are passed to the called function.
        pub end_time: u128,
        pub resulted: bool,
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
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
    pub struct HighestBid {
        /// The `AccountId` of the contract that is called in this transaction.
        pub bidder: AccountId,
        pub bid: Balance,
        /// Gas limit for the execution of the call.
        pub last_bid_time: u128,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct SubAuction {
        /// @notice ERC721 Address -> Token ID -> Auction Parameters
        auctions: Mapping<(AccountId, TokenId), Auction>,
        /// @notice ERC721 Address -> Token ID -> Auction Parameters
        highest_bids: Mapping<(AccountId, TokenId), HighestBid>,
        /// @notice globally and across all auctions, the amount by which a bid has to increase
        min_bid_increment: Balance,
        /// @notice global bid withdrawal lock time20 minutes;
        bid_withdrawal_lock_time: u128,
        /// # note Platform fee
        platform_fee: Balance,
        /// # note Platform fee receipient
        fee_recipient: AccountId,
        address_registry: AccountId,
        is_paused: bool,
        /// contract owner
        owner: AccountId,
    }
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwner,
        ContractPaused,
        NotOwneAndOrContracNotApproved,
        InvalidPayToken,
        AuctionAlreadyStarted,
        EndTimeMustBeGreaterThanStartBy5Minutes,
        InvalidStartTime,
        TransferFailed,
        NoContractsPermitted,
        BiddingOutsideOfTheAuctionWindow,
        ERC20MethodUsedForSUBAuction,
        BidCannotBeLowerThanReservePrice,
        FailedToOutbidHighestBidder,
        InsufficientBalanceOrNotApproved,
        YouAreNotTheHighestBidder,
        CanWithdrawOnlyAfter12HoursAfterAuctionEnded,
        SenderMustBeItemOwner,
        AuctioncNotApproved,
        NoAuctionExists,
        AuctionNotEnded,
        AuctionAlreadyResulted,
        NoOpenBids,
        HighestBidIsBelowReservePrice,
        InsufficientFunds,
        FailedToSendPlatformFee,
        FailedToSendTheOwnerTheirRoyalties,
        FailedToSendTheRoyalties,
        FailedToSendTheOwnerTheAuctionBalance,
        NotOwneAndOrContractNotApproved,
        StartTimeShouldBeLessThanEndTimeBy5Minutes,
        AuctionAlreadyEnded,
        EndTimeMustBeGreaterThanStart,
        AuctionShouldEndAfter5Minutes,
        FailedToRefundPreviousBidder,
        InvalidAddress,
        TransactionFailed,
    }

    // The SubAuction result types.
    pub type Result<T> = core::result::Result<T, Error>;
    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct SubAuctionContractDeployed {}
    #[ink(event)]
    pub struct PauseToggled {
        is_paused: bool,
    }
    /// Event emitted when a token AuctionCreated occurs.
    #[ink(event)]
    pub struct AuctionCreated {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        pay_token: AccountId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct UpdateAuctionEndTime {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        end_time: u128,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct UpdateAuctionStartTime {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        start_time: u128,
    }

    #[ink(event)]
    pub struct UpdateAuctionReservePrice {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        pay_token: AccountId,
        reserve_price: Balance,
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

    /// Event emitted when a token UpdateBidWithdrawalLockTime occurs.
    #[ink(event)]
    pub struct UpdateMinBidIncrement {
        min_bid_increment: Balance,
    }
    /// Event emitted when a token UpdateBidWithdrawalLockTime occurs.
    #[ink(event)]
    pub struct UpdateBidWithdrawalLockTime {
        bid_withdrawal_lock_time: u128,
    }
    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct BidPlaced {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        #[ink(topic)]
        bidder: AccountId,
        bid: Balance,
    }

    /// Event emitted when a token Minted occurs.
    #[ink(event)]
    pub struct BidWithdrawn {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        #[ink(topic)]
        bidder: AccountId,
        bid: Balance,
    }

    /// Event emitted when a token Minted occurs.
    #[ink(event)]
    pub struct BidRefunded {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        #[ink(topic)]
        bidder: AccountId,
        bid: Balance,
    }
    /// Event emitted when a token AuctionResulted occurs.
    #[ink(event)]
    pub struct AuctionResulted {
        old_owner: AccountId,
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
        #[ink(topic)]
        winner: AccountId,
        pay_token: AccountId,
        unit_price: Balance,
        winning_bid: Balance,
    }

    /// Event emitted when a token AuctionCancelled occurs.
    #[ink(event)]
    pub struct AuctionCancelled {
        #[ink(topic)]
        nft_address: AccountId,
        #[ink(topic)]
        token_id: TokenId,
    }
    impl SubAuction {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(fee_recipient: AccountId) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                Self::initialize(contract, fee_recipient)
            })
        }
        /// @notice Contract initializer
        fn initialize(&mut self, fee_recipient: AccountId) {
            assert!(fee_recipient != AccountId::from([0x0; 32]));
            self.owner = Self::env().caller();
            self.fee_recipient = fee_recipient;
            self.platform_fee = 25;
            self.min_bid_increment = 1;
            self.bid_withdrawal_lock_time = 20;
            Self::env().emit_event(SubAuctionContractDeployed {});
        }

        /**
        @notice Creates a new auction for a given item
        @dev Only the owner of item can create an auction and must have approved the contract
        @dev In addition to owning the item, the sender also has to have the MINTER role.
        @dev End time for the auction must be in the future.
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the item being auctioned
        @param _payToken Paying token
        @param _reservePrice Item cannot be sold for less than this or minBidIncrement, whichever is higher
        @param _startTimestamp Unix epoch in seconds for the auction start time
        @param _endTimestamp Unix epoch in seconds for the auction end time.
        */
        #[ink(message)]
        pub fn create_auction(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            reserve_price: Balance,
            start_time: u128,
            min_bid_reserve: bool,
            end_time: u128,
        ) -> Result<()> {
            //whenNotPaused
            ensure!(!self.is_paused, Error::ContractPaused);
            self.ensure_erc721_is_approved_for_all_and_token_registry_enabled(
                self.env().caller(),
                self.env().account_id(),
                nft_address,
                token_id,
                pay_token,
            )?;
            self._create_auction(
                nft_address,
                token_id,
                pay_token,
                reserve_price,
                start_time,
                min_bid_reserve,
                end_time,
            )?;
            Ok(())
        }

        /**
        @notice Places a new bid, out bidding the existing bidder if found and criteria is reached
        @dev Only callable when the auction is open
        @dev Bids from smart contracts are prohibited to prevent griefing with always reverting receiver
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the item being auctioned
        @param _bidAmount Bid amount
        */
        #[ink(message)]
        pub fn place_bid(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            bid_amount: Balance,
        ) -> Result<()> {
            ensure!(!self.is_paused, Error::ContractPaused);
            ensure!(
                !self.env().is_contract(&self.env().caller()),
                Error::NoContractsPermitted
            );
            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            ensure!(
                auction.start_time <= self.get_now() && auction.end_time >= self.get_now(),
                Error::BiddingOutsideOfTheAuctionWindow
            );
            ensure!(
                auction.pay_token != AccountId::from([0x0; 32]),
                Error::ERC20MethodUsedForSUBAuction
            );
            self._place_bid(nft_address, token_id, bid_amount)?;
            Ok(())
        }

        /**
        @notice Allows the hightest bidder to withdraw the bid (after 12 hours post auction's end)
        @dev Only callable by the existing top bidder
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the item being auctioned
        */
        #[ink(message)]
        pub fn withdraw_bid(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            bid_amount: Balance,
        ) -> Result<()> {
            ensure!(!self.is_paused, Error::ContractPaused);
            let highest_bid = self.highest_bids.get(&(nft_address, token_id)).unwrap();
            ensure!(
                highest_bid.bidder == self.env().caller(),
                Error::YouAreNotTheHighestBidder
            );

            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            ensure!(
                auction.end_time < self.get_now() && self.get_now() - auction.end_time >= 43200,
                Error::CanWithdrawOnlyAfter12HoursAfterAuctionEnded
            );
            let previous_bid = highest_bid.bid;
            self.highest_bids.remove(&(nft_address, token_id));
            self._refund_highest_bidder(nft_address, token_id, self.env().caller(), previous_bid)?;
            self.env().emit_event(BidWithdrawn {
                nft_address,
                token_id,
                bidder: self.env().caller(),
                bid: previous_bid,
            });

            Ok(())
        }

        /**
        @notice Closes a finished auction and rewards the highest bidder
        @dev Only admin or smart contract
        @dev Auction can only be resulted if there has been a bidder and reserve met.
        @dev If there have been no bids, the auction needs to be cancelled instead using `cancelAuction()`
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the item being auctioned
        */
        #[ink(message)]
        pub fn result_auction(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            bid_amount: Balance,
        ) -> Result<()> {
            // Check the auction to see if it can be resulted

            let mut auction = self.auctions.get((nft_address, token_id)).unwrap();
            self.ensure_erc721_is_approved_for_all(
                self.env().caller(),
                self.env().account_id(),
                nft_address,
                token_id,
                auction.owner,
            )?;

            // Check the auction real

            ensure!(auction.end_time > 0, Error::NoAuctionExists);
            // Check the auction has ended
            ensure!(auction.end_time < self.get_now(), Error::AuctionNotEnded);
            // Ensure auction not already resulted

            ensure!(!auction.resulted, Error::AuctionAlreadyResulted);
            let highest_bid = self.highest_bids.get(&(nft_address, token_id)).unwrap();
            ensure!(
                highest_bid.bidder == self.env().caller(),
                Error::YouAreNotTheHighestBidder
            );
            let winner = highest_bid.bidder;
            let winning_bid = highest_bid.bid;
            ensure!(winner == AccountId::from([0x0; 32]), Error::NoOpenBids);
            ensure!(
                winning_bid >= auction.reserve_price,
                Error::HighestBidIsBelowReservePrice
            );
            auction.resulted = true;
            self.highest_bids.remove(&(nft_address, token_id));
            let mut pay_amount = winning_bid;
            if winning_bid > auction.reserve_price {
                // Work out total above the reserve
                let above_reserve_price = winning_bid - auction.reserve_price;

                // Work out platform fee from above reserve amount
                let platform_fee_above_reserve = above_reserve_price * self.platform_fee / 1000;

                if auction.pay_token == AccountId::from([0x0; 32]) {
                    // Send platform fee
                    ensure!(
                        platform_fee_above_reserve <= self.env().balance(),
                        Error::InsufficientFunds
                    );
                    ensure!(
                        self.env()
                            .transfer(self.fee_recipient, platform_fee_above_reserve)
                            .is_ok(),
                        Error::FailedToSendPlatformFee
                    );
                } else {
                    ensure!(
                        self.erc20_transfer(
                            auction.pay_token,
                            self.fee_recipient,
                            platform_fee_above_reserve
                        )
                        .is_ok(),
                        Error::FailedToSendPlatformFee
                    );
                }

                // Send remaining to designer
                pay_amount -= platform_fee_above_reserve;
            }
            let (mut minter, mut royalty) =
                self.get_marketplace_minters_royalties(nft_address, token_id)?;
            if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                let royalty_fee = pay_amount * royalty / 10000;
                if auction.pay_token == AccountId::from([0x0; 32]) {
                    // Send platform fee
                    ensure!(
                        royalty_fee <= self.env().balance(),
                        Error::InsufficientFunds
                    );
                    ensure!(
                        self.env().transfer(minter, royalty_fee).is_ok(),
                        Error::FailedToSendTheOwnerTheirRoyalties
                    );
                } else {
                    ensure!(
                        self.erc20_transfer(auction.pay_token, minter, royalty_fee)
                            .is_ok(),
                        Error::FailedToSendTheOwnerTheirRoyalties
                    );
                }
                pay_amount -= royalty_fee;
            } else {
                let (minter, royalty) = self.get_marketplace_collection_royalties(nft_address)?;
                if minter != AccountId::from([0x0; 32]) && royalty != 0 {
                    let royalty_fee = pay_amount * royalty / 10000;
                    if auction.pay_token == AccountId::from([0x0; 32]) {
                        // Send platform fee
                        ensure!(
                            royalty_fee <= self.env().balance(),
                            Error::InsufficientFunds
                        );
                        ensure!(
                            self.env().transfer(minter, royalty_fee).is_ok(),
                            Error::FailedToSendTheRoyalties
                        );
                    } else {
                        ensure!(
                            self.erc20_transfer(auction.pay_token, minter, royalty_fee)
                                .is_ok(),
                            Error::FailedToSendTheRoyalties
                        );
                    }
                    pay_amount -= royalty_fee;
                }
            }
            if pay_amount > 0 {
                if auction.pay_token == AccountId::from([0x0; 32]) {
                    // Send platform fee
                    ensure!(pay_amount <= self.env().balance(), Error::InsufficientFunds);
                    ensure!(
                        self.env().transfer(auction.owner, pay_amount).is_ok(),
                        Error::FailedToSendTheOwnerTheAuctionBalance
                    );
                } else {
                    ensure!(
                        self.erc20_transfer(auction.pay_token, auction.owner, pay_amount)
                            .is_ok(),
                        Error::FailedToSendTheOwnerTheAuctionBalance
                    );
                }
            }
            ensure!(
                self.erc721_transfer_from(
                    nft_address,
                    self.erc721_owner_of(nft_address, token_id)?.unwrap(),
                    winner,
                    token_id
                )
                .is_ok(),
                Error::NotOwneAndOrContractNotApproved
            );
            let mut unit_price = self.get_bundle_marketplace_unit_price(nft_address, token_id)?;

            self.env().emit_event(AuctionResulted {
                old_owner: self.env().caller(),
                nft_address,
                token_id,
                winner,
                pay_token: auction.pay_token,
                unit_price,
                winning_bid,
            });
            self.auctions.remove(&(nft_address, token_id));
            Ok(())
        }

        /**
        @notice Cancels and inflight and un-resulted auctions, returning the funds to the top bidder if found
        @dev Only item owner
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        */
        #[ink(message)]
        pub fn cancel_auction(&mut self, nft_address: AccountId, token_id: TokenId) -> Result<()> {
            // Check valid and not resulted
            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            self.ensure_erc721_owner_of(nft_address, token_id, auction.owner)?;
            // Check the auction real
            ensure!(auction.end_time > 0, Error::NoAuctionExists);
            // Ensure auction not already resulted
            ensure!(!auction.resulted, Error::AuctionAlreadyResulted);
            self._cancel_auction(nft_address, token_id)?;
            Ok(())
        }
        fn _cancel_auction(&mut self, nft_address: AccountId, token_id: TokenId) -> Result<()> {
            let highest_bid = self.highest_bids.get(&(nft_address, token_id)).unwrap();
            if highest_bid.bidder != AccountId::from([0x0; 32]) {
                self._refund_highest_bidder(
                    nft_address,
                    token_id,
                    highest_bid.bidder,
                    highest_bid.bid,
                )?;
                self.highest_bids.remove(&(nft_address, token_id))
            }

            let auction = self.auctions.remove(&(nft_address, token_id));

            self.env().emit_event(AuctionCancelled {
                nft_address,
                token_id,
            });

            Ok(())
        }

        /**
        @notice Toggling the pause flag
        @dev Only admin
        */
        #[ink(message)]
        pub fn toggle_is_paused(&mut self) -> Result<()> {
            ensure!(self.owner == self.env().caller(), Error::OnlyOwner);
            self.is_paused = !self.is_paused;
            self.env().emit_event(PauseToggled {
                is_paused: self.is_paused,
            });

            Ok(())
        }
        /**
        @notice Update the amount by which bids have to increase, across all auctions
        @dev Only admin
        @param _minBidIncrement New bid step in WEI
        */
        #[ink(message)]
        pub fn update_min_bid_increment(&mut self, min_bid_increment: Balance) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.min_bid_increment = min_bid_increment;
            self.env()
                .emit_event(UpdateMinBidIncrement { min_bid_increment });
            Ok(())
        }
        /**
        @notice Update the global bid withdrawal lockout time
        @dev Only admin
        @param _bidWithdrawalLockTime New bid withdrawal lock time
        */
        #[ink(message)]
        pub fn update_bid_withdrawal_lock_time(
            &mut self,
            bid_withdrawal_lock_time: u128,
        ) -> Result<()> {
            //onlyOwner
            ensure!(self.env().caller() == self.owner, Error::OnlyOwner);
            self.bid_withdrawal_lock_time = bid_withdrawal_lock_time;
            self.env().emit_event(UpdateBidWithdrawalLockTime {
                bid_withdrawal_lock_time,
            });
            Ok(())
        }
        /**
        @notice Update the current reserve price for an auction
        @dev Only admin
        @dev Auction must exist
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        @param _reservePrice New Ether reserve price (WEI value)
        */
        #[ink(message)]
        pub fn update_auction_reserve_price(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            reserve_price: u128,
        ) -> Result<()> {
            let mut auction = self.auctions.get(&(nft_address, token_id)).unwrap();

            ensure!(
                self.env().caller() == auction.owner,
                Error::SenderMustBeItemOwner
            );

            // Ensure auction not already resulted
            ensure!(!auction.resulted, Error::AuctionAlreadyResulted);
            // Check the auction real
            ensure!(auction.end_time > 0, Error::NoAuctionExists);
            auction.reserve_price = reserve_price;
            self.auctions.insert(&(nft_address, token_id), &auction);
            self.env().emit_event(UpdateAuctionReservePrice {
                nft_address,
                token_id,
                pay_token: auction.pay_token,
                reserve_price,
            });
            Ok(())
        }

        /**
        @notice Update the current start time for an auction
        @dev Only admin
        @dev Auction must exist
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        @param _startTime New start time (unix epoch in seconds)
        */
        #[ink(message)]
        pub fn update_auction_start_time(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            start_time: u128,
        ) -> Result<()> {
            let mut auction = self.auctions.get(&(nft_address, token_id)).unwrap();

            ensure!(
                self.env().caller() == auction.owner,
                Error::SenderMustBeItemOwner
            );
            ensure!(start_time > 0, Error::InvalidStartTime);
            ensure!(
                auction.start_time + 60 > self.get_now(),
                Error::AuctionAlreadyStarted
            );
            ensure!(
                start_time + 300 < auction.end_time,
                Error::StartTimeShouldBeLessThanEndTimeBy5Minutes
            );
            // Ensure auction not already resulted
            ensure!(!auction.resulted, Error::AuctionAlreadyResulted);
            // Check the auction real
            ensure!(auction.end_time > 0, Error::NoAuctionExists);
            auction.start_time = start_time;
            self.auctions.insert(&(nft_address, token_id), &auction);
            self.env().emit_event(UpdateAuctionStartTime {
                nft_address,
                token_id,
                start_time,
            });
            Ok(())
        }

        /**
        @notice Update the current end time for an auction
        @dev Only admin
        @dev Auction must exist
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        @param _endTimestamp New end time (unix epoch in seconds)
        */
        #[ink(message)]
        pub fn update_auction_end_time(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            end_time: u128,
        ) -> Result<()> {
            let mut auction = self.auctions.get(&(nft_address, token_id)).unwrap();

            ensure!(
                self.env().caller() == auction.owner,
                Error::SenderMustBeItemOwner
            );

            ensure!(
                auction.end_time > self.get_now(),
                Error::AuctionAlreadyEnded
            );
            // Check the auction real
            ensure!(auction.end_time > 0, Error::NoAuctionExists);
            ensure!(
                auction.start_time < end_time,
                Error::EndTimeMustBeGreaterThanStart
            );
            ensure!(
                end_time > self.get_now() + 300,
                Error::AuctionShouldEndAfter5Minutes
            );

            auction.end_time = end_time;
            self.auctions.insert(&(nft_address, token_id), &auction);
            self.env().emit_event(UpdateAuctionEndTime {
                nft_address,
                token_id,
                end_time,
            });
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
            self.env().emit_event(UpdatePlatformFee { platform_fee });
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
        @notice Method for getting all info about the auction
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        */
        #[ink(message)]
        pub fn get_auction(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> (AccountId, AccountId, Balance, Balance, u128, u128, bool) {
            // Check valid and not resulted

            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            (
                auction.owner,
                auction.pay_token,
                auction.min_bid,
                auction.reserve_price,
                auction.start_time,
                auction.end_time,
                auction.resulted,
            )
        }
        #[ink(message)]
        pub fn get_auction_start_time_resulted(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> (u128, bool) {
            // Check valid and not resulted
            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            (auction.start_time, auction.resulted)
        }
        /**
        @notice Method for getting all info about the highest bidder
        @param _tokenId Token ID of the NFT being auctioned
        */
        #[ink(message)]
        pub fn get_highest_bid(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> (AccountId, Balance, u128) {
            let highest_bid = self.highest_bids.get(&(nft_address, token_id)).unwrap();
            (
                highest_bid.bidder,
                highest_bid.bid,
                highest_bid.last_bid_time,
            )
        }

        /**
         * @notice Reclaims ERC20 Compatible tokens for entire balance
         * @dev Only access controls admin
         * @param _tokenContract The address of the token contract
         */
        #[ink(message)]
        pub fn reclaim_erc20(&mut self, token_contract: AccountId) -> Result<()> {
            ensure!(
                token_contract != AccountId::from([0x0; 32]),
                Error::InvalidAddress
            );
            let balance = self.erc20_balance_of(token_contract, self.env().account_id())?;
            ensure!(
                self.erc20_transfer(token_contract, self.env().caller(), balance)
                    .is_ok(),
                Error::TransferFailed
            );
            Ok(())
        }
        fn erc20_balance_of(&mut self, token: AccountId, owner: AccountId) -> Result<Balance> {
            let mut ans = Balance::default();
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0x0F, 0x75, 0x5A, 0x56]; //erc20 balance_of 
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
                ans = result?;
            }
            Ok(ans)
        }
    }
    #[ink(impl)]
    impl SubAuction {
        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn auction_of_impl(&self, owner: &AccountId, token_id: TokenId) -> Auction {
            self.auctions.get((owner, token_id)).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn highest_bid_impl(&self, owner: &AccountId, token_id: &TokenId) -> HighestBid {
            self.highest_bids.get((owner, token_id)).unwrap_or_default()
        }
        /**
        @notice Private method doing the heavy lifting of creating an auction
        @param _nftAddress ERC 721 Address
        @param _tokenId Token ID of the NFT being auctioned
        @param _payToken Paying token
        @param _reservePrice Item cannot be sold for less than this or minBidIncrement, whichever is higher
        @param _startTimestamp Unix epoch in seconds for the auction start time
        @param _endTimestamp Unix epoch in seconds for the auction end time.
        */
        fn _create_auction(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
            reserve_price: Balance,
            start_time: u128,
            min_bid_reserve: bool,
            end_time: u128,
        ) -> Result<()> {
            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            ensure!(auction.end_time == 0, Error::AuctionAlreadyStarted);
            ensure!(
                end_time >= start_time + 300,
                Error::EndTimeMustBeGreaterThanStartBy5Minutes
            );
            ensure!(start_time > self.get_now(), Error::InvalidStartTime);
            let mut min_bid = 0;
            if min_bid_reserve {
                min_bid = reserve_price;
            }
            self.auctions.insert(
                &(nft_address, token_id),
                &Auction {
                    owner: self.env().caller(),
                    pay_token,
                    min_bid,
                    reserve_price,
                    start_time,
                    end_time,
                    resulted: false,
                },
            );
            // Send NativeToken fee to fee recipient
            ensure!(
                self.env()
                    .transfer(self.fee_recipient, self.env().transferred_value())
                    .is_ok(),
                Error::TransferFailed
            );
            self.env().emit_event(AuctionCreated {
                nft_address,
                token_id,
                pay_token,
            });
            Ok(())
        }

        fn _place_bid(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            bid_amount: Balance,
        ) -> Result<()> {
            ensure!(!self.is_paused, Error::ContractPaused);

            let auction = self.auctions.get((nft_address, token_id)).unwrap();
            if auction.min_bid == auction.reserve_price {
                ensure!(
                    auction.reserve_price <= bid_amount,
                    Error::BidCannotBeLowerThanReservePrice
                );
            }
            let mut highest_bid = self.highest_bids.get((nft_address, token_id)).unwrap();
            let min_bid_required = highest_bid.bid + self.min_bid_increment;
            ensure!(
                min_bid_required <= bid_amount,
                Error::FailedToOutbidHighestBidder
            );
            if auction.pay_token != AccountId::from([0x0; 32]) {
                ensure!(
                    self.erc20_transfer_from(
                        auction.pay_token,
                        self.env().caller(),
                        self.env().account_id(),
                        bid_amount
                    )
                    .is_ok(),
                    Error::FailedToOutbidHighestBidder
                );
            }
            if highest_bid.bidder != AccountId::from([0x0; 32]) {
                self._refund_highest_bidder(
                    nft_address,
                    token_id,
                    highest_bid.bidder,
                    highest_bid.bid,
                )?;
            }
            highest_bid.bidder = self.env().caller();
            highest_bid.bid = bid_amount;
            highest_bid.last_bid_time = self.get_now();
            self.env().emit_event(BidPlaced {
                nft_address,
                token_id,
                bidder: self.env().caller(),
                bid: bid_amount,
            });

            Ok(())
        }
        fn get_now(&self) -> u128 {
            self.env().block_timestamp().into()
        }
        /**
        @notice Used for sending back escrowed funds from a previous bid
        @param _currentHighestBidder Address of the last highest bidder
        @param _currentHighestBid Ether or Mona amount in WEI that the bidder sent when placing their bid
        */
        fn _refund_highest_bidder(
            &mut self,
            nft_address: AccountId,
            token_id: TokenId,
            current_highest_bidder: AccountId,
            current_highest_bid: Balance,
        ) -> Result<()> {
            let auction = self.auctions.get((nft_address, token_id)).unwrap();

            if auction.pay_token == AccountId::from([0x0; 32]) {
                // Send platform fee
                ensure!(
                    current_highest_bid <= self.env().balance(),
                    Error::InsufficientFunds
                );
                ensure!(
                    self.env()
                        .transfer(current_highest_bidder, current_highest_bid)
                        .is_ok(),
                    Error::FailedToRefundPreviousBidder
                );
            } else {
                ensure!(
                    self.erc20_transfer(
                        auction.pay_token,
                        current_highest_bidder,
                        current_highest_bid
                    )
                    .is_ok(),
                    Error::FailedToRefundPreviousBidder
                );
            }
            self.env().emit_event(BidRefunded {
                nft_address,
                token_id,
                bidder: current_highest_bidder,
                bid: current_highest_bid,
            });
            Ok(())
        }

        fn ensure_erc721_is_approved_for_all_and_token_registry_enabled(
            &self,
            owner: AccountId,
            operator: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
            pay_token: AccountId,
        ) -> Result<()> {
            ensure!(
                Some(self.env().caller()) == self.erc721_owner_of(nft_address, token_id)?
                    && self.erc721_is_approved_for_all(nft_address, owner, operator)?,
                Error::NotOwneAndOrContractNotApproved
            );
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);

                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.token_registry(),
                    Error::InvalidPayToken
                );
                ensure!(
                    self.token_registry_enabled(
                        address_registry_instance.token_registry(),
                        pay_token
                    )?,
                    Error::InvalidPayToken,
                );
            }
            Ok(())
        }
        #[cfg_attr(test, allow(unused_variables))]
        fn token_registry_enabled(&self, callee: AccountId, token: AccountId) -> Result<bool> {
            let mut ans = false;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
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
                ans = result?;
            }
            Ok(ans)
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
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
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

        fn erc20_transfer_from(
            &mut self,
            token: AccountId,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
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
        fn erc20_transfer(
            &mut self,
            token: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0x84, 0xA1, 0x5D, 0xA1]; //erc20 transfer
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
            let mut ans = false;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
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
                ans = result?;
            }
            Ok(ans)
        }

        fn erc721_owner_of(
            &self,
            token: AccountId,
            token_id: TokenId,
        ) -> Result<Option<AccountId>> {
            let mut ans = None;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
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
                ans = result?;
            }
            Ok(ans)
        }

        fn ensure_erc721_is_approved_for_all(
            &self,
            owner: AccountId,
            operator: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
            auction_owner: AccountId,
        ) -> Result<()> {
            ensure!(
                Some(self.env().caller()) == self.erc721_owner_of(nft_address, token_id)?
                    && self.env().caller() == auction_owner,
                Error::SenderMustBeItemOwner
            );

            ensure!(
                self.erc721_is_approved_for_all(nft_address, owner, operator)
                    .is_ok(),
                Error::AuctioncNotApproved
            );
            Ok(())
        }
        fn ensure_erc721_owner_of(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
            auction_owner: AccountId,
        ) -> Result<()> {
            ensure!(
                Some(self.env().caller()) == self.erc721_owner_of(nft_address, token_id)?
                    && self.env().caller() == auction_owner,
                Error::SenderMustBeItemOwner
            );
            Ok(())
        }
        fn get_marketplace_minters_royalties(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<(AccountId, Balance)> {
            let mut ans = (AccountId::from([0x0; 32]), 0);
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                let minter = self.marketplace_minters(
                    address_registry_instance.marketplace(),
                    nft_address,
                    token_id,
                )?;
                let royalty = self.marketplace_royalties(
                    address_registry_instance.marketplace(),
                    nft_address,
                    token_id,
                )?;
                ans = (minter, royalty);
            }
            Ok(ans)
        }
        fn marketplace_minters(
            &self,
            token: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<AccountId> {
            let mut ans = AccountId::from([0x0; 32]);
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0x4B, 0x4C, 0x7E, 0xC9]; //marketplace_minter_of
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
                            .push_arg(token_id),
                    )
                    .returns::<AccountId>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ans = result?;
            }
            Ok(ans)
        }

        fn marketplace_royalties(
            &self,
            token: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<Balance> {
            let mut ans = 0;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0x13, 0x5B, 0x72, 0xF2]; //marketplace royalty_of 
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
                            .push_arg(token_id),
                    )
                    .returns::<Balance>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ans = result?;
            }
            Ok(ans)
        }
        fn get_marketplace_collection_royalties(
            &self,
            nft_address: AccountId,
        ) -> Result<(AccountId, Balance)> {
            let mut ans = (AccountId::from([0x0; 32]), 0);
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                let (minter, royalty) = self.marketplace_collection_royalties(
                    address_registry_instance.marketplace(),
                    nft_address,
                )?;
                ans = (minter, royalty);
            }
            Ok(ans)
        }
        fn marketplace_collection_royalties(
            &self,
            token: AccountId,
            nft_address: AccountId,
        ) -> Result<(AccountId, Balance)> {
            let mut ans = (AccountId::from([0x0; 32]), 0);
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0xAA, 0xFC, 0xD7, 0xEA]; //marketplace_collection_royalties 
                let (gas_limit, transferred_value) = (0, 0);
                let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                    .call_type(
                        Call::new()
                            .callee(token)
                            .gas_limit(gas_limit)
                            .transferred_value(transferred_value),
                    )
                    .exec_input(ExecutionInput::new(selector.into()).push_arg(nft_address))
                    .returns::<(AccountId, Balance)>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ans = result?;
            }
            Ok(ans)
        }
        fn get_bundle_marketplace_unit_price(
            &self,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<Balance> {
            let mut ans = Balance::default();
            #[cfg(not(test))]
            {
                let address_registry_instance: sub_address_registry::SubAddressRegistryRef =
                    ink_env::call::FromAccountId::from_account_id(self.address_registry);
                ensure!(
                    AccountId::from([0x0; 32]) == address_registry_instance.bundle_marketplace(),
                    Error::InvalidPayToken
                );
                self.bundle_marketplace_validate_item_sold(
                    address_registry_instance.bundle_marketplace(),
                    nft_address,
                    token_id,
                )?;

                let unit_price = self
                    .marketplace_get_price(address_registry_instance.marketplace(), nft_address)?;
                ans = unit_price;
            }

            Ok(ans)
        }
        fn bundle_marketplace_validate_item_sold(
            &self,
            token: AccountId,
            nft_address: AccountId,
            token_id: TokenId,
        ) -> Result<()> {
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0x5E, 0x38, 0x31, 0x94]; //bundle_marketplace_validate_item_sold 
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
                            .push_arg(1),
                    )
                    .returns::<(AccountId, Balance)>()
                    .fire()
                    .map_err(|_| Error::TransactionFailed);
                ensure!(result.is_ok(), Error::TransactionFailed);
            }
            Ok(())
        }
        fn marketplace_get_price(
            &self,
            token: AccountId,
            nft_address: AccountId,
        ) -> Result<Balance> {
            let mut ans = 0;
            #[cfg(not(test))]
            {
                use ink_env::call::{build_call, Call, ExecutionInput, Selector};
                let selector: [u8; 4] = [0xF2, 0x3D, 0x4B, 0x6C]; //marketplace_get_price  
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
                ans = result?;
            }
            Ok(ans)
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
