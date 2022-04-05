#![cfg_attr(not(feature = "std"), no_std)]

/// This pallet was forked from https://github.com/OAK-Foundation/quadratic-funding-pallet-original
/// It's the base for the funding-pallet of fair-squares

pub use pallet::*;


use frame_support::{
	pallet_prelude::*,
	codec::{Decode, Encode},
	traits::{ReservableCurrency, ExistenceRequirement, Currency, WithdrawReasons,Get
	}
};

use frame_system::{ensure_signed, ensure_root};
use sp_runtime::{
	traits::{AccountIdConversion},
	ModuleId,
};

use sp_std::prelude::*;
use sp_std::{convert::{TryInto}};

#[frame_support::pallet]
pub mod pallet {
	#[pallet::config]
    pub trait Config: frame_system::Config {
    
	#[pallet::constant]
    type PalletId: Get<PalletId>;

	/// The currency in which the housefund will be denominated
	type Currency: ReservableCurrency<Self::AccountId>;

    pub type HousingFund = u32;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    type FundOf<T> = HousingFund<AccountIdOf<T>, <T as frame_system::Config>::BlockNumber>;
    type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>>;
	}

    #[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
    pub trait Config: frame_system::Config {
	// used to generate sovereign account
    


    // Grant in round
    #[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
    pub struct HousingFund<AccountId, Balance, BlockNumber> {
        contributions: Vec<Contribution<AccountId, Balance, BlockNumber>>,
        is_withdrawn: bool,
        fund: Balance,
    }

    /// The contribution users made to a grant Fund.
    #[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
    pub struct Contribution<AccountId, Balance,BlockNumber> {
        account_id: AccountId,
        value: Balance,
		blocknumber: BlockNumber,
    }



	#[pallet::storage]
	#[pallet::getter(fn)]
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		FundCount get(fn fund):HousingFund;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T> where Balance = BalanceOf<T>, AccountId = <T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber {
		ContributeSucceed(AccountId, Balance, BlockNumber),
		Contribution(),	
		FundSucceed(),
	}


    #[pallet::error]
    pub enum Error <T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// There was an overflow.
		Overflow,
		///
		InvalidAccount,
		StartBlockNumberTooSmall,
		RoundNotProcessing,
		RoundCanceled,
		RoundFinalized,
		RoundNotFinalized,
		GrantAmountExceed,
		WithdrawalExpirationExceed,
		NotEnoughFund,
		InvalidFundIndexes,
	}


    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		/// Contribute a grant
		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,1)]
		pub fn contribute(origin, Fund_index: FundIndex, value: BalanceOf<T>) {
			let who = ensure_signed(origin)?;
			ensure!(value > (0 as u32).into(), Error::<T>::InvalidParam);
			let Fund_count = FundCount::get();
			ensure!(Fund_index < Fund_count, Error::<T>::InvalidParam);
			let now = <frame_system::Module<T>>::block_number();
			

			match found_contribution {
				Some(contribution) => {
					contribution.value += value;
				},
				None => {
					grant.contributions.push(ContributionOf::<T> {
						account_id: who.clone(),
						value: value,
					});
				}
			}

			// Transfer contribute to grant account
			<T as Config>::Currency::transfer(
				&who,
				&Self::Fund_account_id(Fund_index),
				value,
				ExistenceRequirement::AllowDeath
			)?;
			
			Self::deposit_event(RawEvent::ContributeSucceed(who, Fund_index, value, now));
		}


		/// Withdraw
		#[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
		pub fn withdraw(origin, round_index: RoundIndex, Fund_index: FundIndex) {
			let who = ensure_signed(origin)?;
			let now = <frame_system::Module<T>>::block_number();

			// Distribute contribution amount
			let _ = <T as Config>::Currency::resolve_into_existing(&Fund.owner, <T as Config>::Currency::withdraw(
				&Self::Fund_account_id(Fund_index),
				contribution_amount,
				WithdrawReasons::from(WithdrawReasons::TRANSFER),
				ExistenceRequirement::AllowDeath,
			)?);

			// Set is_withdrawn
			grant.is_withdrawn = true;
			grant.withdrawal_expiration = now + <WithdrawalExpiration<T>>::get();

			<Rounds<T>>::insert(round_index, round.clone());

			Self::deposit_event(RawEvent::GrantWithdrawn(round_index, Fund_index, matching_fund, contribution_amount));
		}


impl<T: Config> Module<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		return T::PalletId::get().into_account();
	}

	
	// Calculate used funds
	pub fn get_used_fund() -> BalanceOf<T> {
		let now = <frame_system::Module<T>>::block_number();
		let mut used_fund: BalanceOf<T> = (0 as u32).into();


				used_fund += grant.matching_fund;
			}
		}

		used_fund
	}
}

impl<AccountId, Balance: From<u32>, BlockNumber: From<u32>> Round<AccountId, Balance, BlockNumber> {
	fn new(start: BlockNumber, end: BlockNumber, matching_fund: Balance, Fund_indexes: Vec<FundIndex>) -> Round<AccountId, Balance, BlockNumber> { 
		let mut grant_round  = Round {
			start: start,
			end: end,
			owner: BoundedVec<owner>,
			amount: amount,
			is_canceled: false,
			is_finalized: false,
			nft:nft
		};
	fn select_contributor(){};

		//TODO:logic of fair contribution
		
		
	}
	grant_round
}