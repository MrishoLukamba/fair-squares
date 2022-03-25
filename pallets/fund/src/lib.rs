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
    
	#[pallet::constant]
    type PalletId: Get<PalletId>;

	/// The currency in which the housefund will be denominated
	type Currency: ReservableCurrency<Self::AccountId>;}

    pub type HousingFund = u32;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    type FundOf<T> = HousingFund<AccountIdOf<T>, <T as frame_system::Config>::BlockNumber>;
    type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;


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
		Contribution
		GrantApproved(RoundIndex, FundIndex),
		RoundCanceled(RoundIndex),
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

		/// Create Fund
		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,2)]
		pub fn create_Fund(origin, name: Vec<u8>, logo: Vec<u8>, description: Vec<u8>, website: Vec<u8>) {
			let who = ensure_signed(origin)?;

			
			let index = FundCount::get();
			let next_index = index.checked_add(1).ok_or(Error::<T>::Overflow)?;

			// Add grant to list
			<Funds<T>>::insert(index, Fund);
			FundCount::put(next_index);

			Self::deposit_event(RawEvent::FundCreated(index));
		}

		/// Funding to matching fund pool
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn fund(origin, fund_balance: BalanceOf<T>) {
			let who = ensure_signed(origin)?;
			ensure!(fund_balance > (0 as u32).into(), Error::<T>::InvalidParam);

			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			<T as Config>::Currency::resolve_creating(&Self::account_id(), <T as Config>::Currency::withdraw(
				&who,
				fund_balance,
				WithdrawReasons::from(WithdrawReasons::TRANSFER),
				ExistenceRequirement::AllowDeath,
			)?);

			Self::deposit_event(RawEvent::FundSucceed());
		}


		/// Contribute a grant
		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,1)]
		pub fn contribute(origin, Fund_index: FundIndex, value: BalanceOf<T>) {
			let who = ensure_signed(origin)?;
			ensure!(value > (0 as u32).into(), Error::<T>::InvalidParam);
			let Fund_count = FundCount::get();
			ensure!(Fund_index < Fund_count, Error::<T>::InvalidParam);
			let now = <frame_system::Module<T>>::block_number();
			
			// round list must be not none
			let round_index = RoundCount::get();
			ensure!(round_index > 0, Error::<T>::NoActiveRound);

			// Find processing round
			let mut processing_round: Option<RoundOf::<T>> = None;
			for i in (0..round_index).rev() {
				let round = <Rounds<T>>::get(i).unwrap();
				if !round.is_canceled && round.start < now && round.end > now {
					processing_round = Some(round);
				}
			}

			let mut round = processing_round.ok_or(Error::<T>::RoundNotProcessing)?;

			// Find grant by index
			let mut found_grant: Option<&mut GrantOf::<T>> = None;
			for grant in round.grants.iter_mut() {
				if grant.Fund_index == Fund_index {
					found_grant = Some(grant);
					break;
				}
			}

			let grant = found_grant.ok_or(Error::<T>::NoActiveGrant)?;
			ensure!(!grant.is_canceled, Error::<T>::GrantCanceled);

			// Find previous contribution by account_id
			// If you have contributed before, then add to that contribution. Otherwise join the list.
			let mut found_contribution: Option<&mut ContributionOf::<T>> = None;
			for contribution in grant.contributions.iter_mut() {
				if contribution.account_id == who {
					found_contribution = Some(contribution);
					break;
				}
			}

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
			
			<Rounds<T>>::insert(round_index-1, round);

			Self::deposit_event(RawEvent::ContributeSucceed(who, Fund_index, value, now));
		}


		/// Withdraw
		#[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
		pub fn withdraw(origin, round_index: RoundIndex, Fund_index: FundIndex) {
			let who = ensure_signed(origin)?;
			let now = <frame_system::Module<T>>::block_number();

			// Only Fund owner can withdraw
			let Fund = Funds::<T>::get(Fund_index).ok_or(Error::<T>::NoActiveGrant)?;
			ensure!(who == Fund.owner, Error::<T>::InvalidAccount);

			let mut round = <Rounds<T>>::get(round_index).ok_or(Error::<T>::NoActiveRound)?;
			let mut found_grant: Option<&mut GrantOf::<T>> = None;
			for grant in round.grants.iter_mut() {
				if grant.Fund_index == Fund_index {
					found_grant = Some(grant);
					break;
				}
			}

			let grant = found_grant.ok_or(Error::<T>::NoActiveGrant)?;
			ensure!(now <= grant.withdrawal_expiration, Error::<T>::WithdrawalExpirationExceed);

			// This grant must not have distributed funds
			ensure!(grant.is_approved, Error::<T>::GrantNotApproved);
			ensure!(!grant.is_withdrawn, Error::<T>::GrantWithdrawn);

			// Calculate contribution amount
			let mut contribution_amount: BalanceOf<T>  = (0 as u32).into();
			for contribution in grant.contributions.iter() {
				let contribution_value = contribution.value;
				contribution_amount += contribution_value;
			}

			let matching_fund = grant.matching_fund;

			// Return funds to caller without charging a transfer fee
			let _ = <T as Config>::Currency::resolve_into_existing(&Fund.owner, <T as Config>::Currency::withdraw(
				&Self::account_id(),
				matching_fund,
				WithdrawReasons::from(WithdrawReasons::TRANSFER),
				ExistenceRequirement::AllowDeath,
			)?);

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


		/// Set max grant count per round
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn set_max_grant_count_per_round(origin, max_grant_count_per_round: u32) {
			ensure_root(origin)?;
			ensure!(max_grant_count_per_round > 0, Error::<T>::InvalidParam);
			MaxGrantCountPerRound::put(max_grant_count_per_round);
		}

		/// Set withdrawal expiration
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn set_withdrawal_expiration(origin, withdrawal_expiration: T::BlockNumber) {
			ensure_root(origin)?;
			ensure!(withdrawal_expiration > (0 as u32).into(), Error::<T>::InvalidParam);
			<WithdrawalExpiration<T>>::put(withdrawal_expiration);
		}


impl<T: Config> Module<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		return T::PalletId::get().into_account();
	}

	pub fn Fund_account_id(index: ) -> T::AccountId {
		T::ModuleId::get().into_sub_account(index)
	}

	/// Get all Funds
	pub fn get_Funds() -> Vec<Fund<AccountIdOf<T>, T::BlockNumber>> {
		let len = FundCount::get();
		let mut Funds: Vec<Fund<AccountIdOf<T>, T::BlockNumber>> = Vec::new();
		for i in 0..len {
			let Fund = <Funds<T>>::get(i).unwrap();
			Funds.push(Fund);
		}
		Funds
	}

	// Calculate used funds
	pub fn get_used_fund() -> BalanceOf<T> {
		let now = <frame_system::Module<T>>::block_number();
		let mut used_fund: BalanceOf<T> = (0 as u32).into();
		let count = RoundCount::get();

		for i in 0..count {
			let round = <Rounds<T>>::get(i).unwrap();

			// The cancelled round does not occupy funds
			if round.is_canceled {
				continue;
			}

			let grants = &round.grants;

			// Rounds that are not finalized always occupy funds
			if !round.is_finalized {
				used_fund += round.matching_fund;
				continue;
			}

			for grant in grants.iter() {
				// If the undrawn funds expire, they will be returned to the foundation.
				if grant.is_approved && !grant.is_withdrawn && grant.withdrawal_expiration > now {
					continue;
				}

				// Because the funds that have been withdrawn are no longer in the foundation account, they will not be recorded.
				if grant.is_withdrawn {
					continue;
				}

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
			matching_fund: matching_fund,
			grants: Vec::new(),
			is_canceled: false,
			is_finalized: false,
		};

		for HousingFund in HousingFund {
			grant_round.grants.push(Grant {
				Fund_index: Fund_index,
				contributions: Vec::new(),
				withdrawal_expiration: (0 as u32).into(),
				matching_fund: (0 as u32).into(),
			});
		}
	}
	grant_round
}