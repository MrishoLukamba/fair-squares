#![cfg_attr(not(feature = "std"), no_std)]

/// This pallet was forked from https://github.com/OAK-Foundation/quadratic-funding-pallet-original
/// It's the base for the funding-pallet of fair-squares

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        codec::{Decode, Encode},
        traits::{ReservableCurrency, ExistenceRequirement, Currency, WithdrawReasons,Get
        };

    use frame_system::{ensure_signed, ensure_root};
    use sp_runtime::{
        traits::{AccountIdConversion},
        ModuleId,
    };

    use sp_std::prelude::*;
    use sp_std::{convert::{TryInto}};

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	// used to generate sovereign account
	// refer: https://github.com/paritytech/substrate/blob/743accbe3256de2fc615adcaa3ab03ebdbbb4dbd/frame/treasury/src/lib.rs#L92
	type ModuleId: Get<ModuleId>;

	/// The currency in which the housefund will be denominated
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

pub type Fund = u32;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type FundOf<T> = Fund<AccountIdOf<T>, <T as frame_system::Config>::BlockNumber>;
type ContributionOf<T> = Contribution<AccountIdOf<T>, BalanceOf<T>>;
type GrantOf<T> = Grant<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;


// Grant in round
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Grant<AccountId, Balance, BlockNumber> {
	Fund_index: FundIndex,
	contributions: Vec<Contribution<AccountId, Balance>>,
	is_approved: bool,
	is_canceled: bool,
	is_withdrawn: bool,
	withdrawal_expiration: BlockNumber,
	matching_fund: Balance,
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

		// Fill in the grants structure in advance
		for Fund_index in Fund_indexes {
			grant_round.grants.push(Grant {
				Fund_index: Fund_index,
				contributions: Vec::new(),
				is_approved: false,
				is_canceled: false,
				is_withdrawn: false,
				withdrawal_expiration: (0 as u32).into(),
				matching_fund: (0 as u32).into(),
			});
		}

		grant_round
	}
} 

/// The contribution users made to a grant Fund.
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Contribution<AccountId, Balance> {
	account_id: AccountId,
	value: Balance,
}

/// Fund struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize))]
pub struct Fund<AccountId, BlockNumber> {
	name: Vec<u8>,
	logo: Vec<u8>,
	description: Vec<u8>,
	website: Vec<u8>,
	/// The account that will receive the funds if the campaign is successful
	owner: AccountId,
	create_block_number: BlockNumber,
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as OpenGrant {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Funds get(fn grants): map hasher(blake2_128_concat) FundIndex => Option<FundOf<T>>;
		FundCount get(fn Fund_count): FundIndex;

		Rounds get(fn rounds): map hasher(blake2_128_concat) RoundIndex => Option<RoundOf<T>>;
		RoundCount get(fn round_count): RoundIndex;

		MaxGrantCountPerRound get(fn max_grant_count_per_round) config(init_max_grant_count_per_round): u32;
		WithdrawalExpiration get(fn withdrawal_expiration) config(init_withdrawal_expiration): T::BlockNumber;

		IsIdentityRequired get(fn is_identity_required) config(init_is_identity_required): bool;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where Balance = BalanceOf<T>, AccountId = <T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		FundCreated(FundIndex),
		RoundCreated(RoundIndex),
		ContributeSucceed(AccountId, FundIndex, Balance, BlockNumber),
		GrantCanceled(RoundIndex, FundIndex),
		GrantWithdrawn(RoundIndex, FundIndex, Balance, Balance),
		GrantApproved(RoundIndex, FundIndex),
		RoundCanceled(RoundIndex),
		FundSucceed(),
		RoundFinalized(RoundIndex),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// There was an overflow.
		Overflow,
		///
		RoundStarted,
		RoundNotEnded,
		StartBlockNumberInvalid,
		EndBlockNumberInvalid,
		EndTooEarly,
		NoActiveRound,
		NoActiveGrant,
		InvalidParam,
		GrantCanceled,
		GrantWithdrawn,
		GrantApproved,
		GrantNotApproved,
		InvalidAccount,
		IdentityNeeded,
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
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// Create Fund
		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,2)]
		pub fn create_Fund(origin, name: Vec<u8>, logo: Vec<u8>, description: Vec<u8>, website: Vec<u8>) {
			let who = ensure_signed(origin)?;

			// Check if identity is required
			let is_identity_needed = IsIdentityRequired::get();
			if is_identity_needed {
				let identity = pallet_identity::Module::<T>::identity(who.clone()).ok_or(Error::<T>::IdentityNeeded)?;
				let mut is_found_judgement = false;
				for judgement in identity.judgements.iter() {
					if judgement.1 == pallet_identity::Judgement::Reasonable || judgement.1 == pallet_identity::Judgement::KnownGood {
						is_found_judgement = true;
						break;
					}
				}
				ensure!(is_found_judgement, Error::<T>::IdentityNeeded);
			}

			// Validation
			ensure!(name.len() > 0, Error::<T>::InvalidParam);
			ensure!(logo.len() > 0, Error::<T>::InvalidParam);
			ensure!(description.len() > 0, Error::<T>::InvalidParam);
			ensure!(website.len() > 0, Error::<T>::InvalidParam);
			
			let index = FundCount::get();
			let next_index = index.checked_add(1).ok_or(Error::<T>::Overflow)?;

			// Create a grant 
			let Fund = FundOf::<T> {
				name: name,
				logo: logo,
				description: description,
				website: website,
				owner: who,
				create_block_number: <frame_system::Module<T>>::block_number(),
			};

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

			// Transfer matching fund to module account
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

		/// Schedule a round
		/// grant_indexes: the grants were selected for this round
		#[weight = 10_000 + T::DbWeight::get().reads_writes(4,3)]
		pub fn schedule_round(origin, start: T::BlockNumber, end: T::BlockNumber, matching_fund: BalanceOf<T>, Fund_indexes: Vec<FundIndex>) {
			ensure_root(origin)?;
			let now = <frame_system::Module<T>>::block_number();

			// Check whether the funds are sufficient
			let used_fund = Self::get_used_fund();
			let free_balance = <T as Config>::Currency::free_balance(&Self::account_id());

			ensure!(free_balance - used_fund >= matching_fund, Error::<T>::NotEnoughFund);

			ensure!(Fund_indexes.len() > 0, Error::<T>::InvalidFundIndexes);
			// The number of items cannot exceed the maximum
			ensure!(Fund_indexes.len() <= MaxGrantCountPerRound::get().try_into().unwrap(), Error::<T>::GrantAmountExceed);
			// The end block must be greater than the start block
			ensure!(end > start, Error::<T>::EndTooEarly);
			// Both the starting block number and the ending block number must be greater than the current number of blocks
			ensure!(start > now, Error::<T>::StartBlockNumberInvalid);
			ensure!(end > now, Error::<T>::EndBlockNumberInvalid);

			// Fund_index should be smaller than Fund count
			let Fund_count = FundCount::get();
			for Fund_index in Fund_indexes.iter() {
				ensure!(*Fund_index < Fund_count, Error::<T>::InvalidFundIndexes);
			}

			// Find the last valid round
			let mut last_valid_round: Option<RoundOf::<T>> = None;
			let index = RoundCount::get();
			for _i in (0..index).rev() {
				let round = <Rounds<T>>::get(index-1).unwrap();
				if !round.is_canceled {
					last_valid_round = Some(round);
					break;
				}
			}

			// The start time must be greater than the end time of the last valid round
			match last_valid_round {
				Some(round) => {
					ensure!(start > round.end, Error::<T>::StartBlockNumberTooSmall);
				},
				None => {}
			}

			let next_index = index.checked_add(1).ok_or(Error::<T>::Overflow)?;

			let round = RoundOf::<T>::new(start, end, matching_fund, Fund_indexes);

			// Add grant round to list
			<Rounds<T>>::insert(index, round);
			RoundCount::put(next_index);

			Self::deposit_event(RawEvent::RoundCreated(index));
		}

		/// Cancel a round
		/// This round must have not started yet
		#[weight = 10_000 + T::DbWeight::get().reads_writes(3,2)]
		pub fn cancel_round(origin, round_index: RoundIndex) {
			ensure_root(origin)?;
			let now = <frame_system::Module<T>>::block_number();
			let count = RoundCount::get();
			let mut round = <Rounds<T>>::get(round_index).ok_or(Error::<T>::NoActiveRound)?;

			// Ensure current round is not started
			ensure!(round.start > now, Error::<T>::RoundStarted);
			// This round cannot be cancelled
			ensure!(!round.is_canceled, Error::<T>::RoundCanceled);

			round.is_canceled = true;
			<Rounds<T>>::insert(round_index, round.clone());

			Self::deposit_event(RawEvent::RoundCanceled(count-1));
		}


			round.is_finalized = true;
			<Rounds<T>>::insert(round_index, round.clone());

			Self::deposit_event(RawEvent::RoundFinalized(round_index));
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

		/// Approve Fund
		/// If the Fund is approve, the Fund owner can withdraw funds
		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,1)]
		pub fn approve(origin, round_index: RoundIndex, Fund_index: FundIndex) {
			ensure_root(origin)?;
			let mut round = <Rounds<T>>::get(round_index).ok_or(Error::<T>::NoActiveRound)?;
			ensure!(round.is_finalized, Error::<T>::RoundNotFinalized);
			ensure!(!round.is_canceled, Error::<T>::RoundCanceled);
			let grants = &mut round.grants;

			// The round must have ended
			let now = <frame_system::Module<T>>::block_number();
			// This round must be over
			ensure!(round.end < now, Error::<T>::RoundNotEnded);

			// Find grant from list
			let mut found_grant: Option<&mut GrantOf::<T>> = None;
			for grant in grants.iter_mut() {
				if grant.Fund_index == Fund_index {
					found_grant = Some(grant);
					break;
				}
			}
			let mut grant = found_grant.ok_or(Error::<T>::NoActiveGrant)?;

			// Can't let users vote in the cancered round
			ensure!(!grant.is_canceled, Error::<T>::GrantCanceled);
			ensure!(!grant.is_approved, Error::<T>::GrantApproved);

			// set is_approved
			grant.is_approved = true;
			grant.withdrawal_expiration = now + <WithdrawalExpiration<T>>::get();

			<Rounds<T>>::insert(round_index, round.clone());

			Self::deposit_event(RawEvent::GrantApproved(round_index, Fund_index));
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

			// Distribute CLR amount
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

		/// set is_identity_required
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn set_is_identity_required(origin, is_identity_required: bool) {
			ensure_root(origin)?;
			IsIdentityRequired::put(is_identity_required);
		}
	}
}

impl<T: Config> Module<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		return T::ModuleId::get().into_account();
	}

	pub fn Fund_account_id(index: FundIndex) -> T::AccountId {
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