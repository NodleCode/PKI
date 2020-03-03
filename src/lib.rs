#![cfg_attr(not(feature = "std"), no_std)]

//! This module implements a Token Curated Registry where members (represented by their
//! `AccountId`) are accepted based on the number of tokens staked in support to their
//! application.

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{result::Result, DispatchResult},
    ensure,
    traits::{ChangeMembers, Currency, Get, LockIdentifier, LockableCurrency, WithdrawReasons},
    Parameter,
};
use sp_runtime::{
    traits::{CheckedAdd, Dispatchable, EnsureOrigin},
    DispatchError, Perbill,
};
use sp_std::prelude::Vec;
use system::ensure_signed;

const TCR_LOCK_ID: LockIdentifier = *b"tcrstake";

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Application<AccountId, Balance> {
    candidate: AccountId,
    candidate_deposit: Balance,
    metadata: Vec<u8>,

    challenger: Option<AccountId>,
    challenger_deposit: Option<Balance>,

    votes_for: Option<Balance>,
    votes_against: Option<Balance>,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency used to represent the voting power
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    /// Minimum amount of tokens required to apply
    type MinimumApplicationAmount: Get<BalanceOf<Self>>;
    /// Minimum amount of tokens required to challenge an entry
    type MinimumChallengeAmount: Get<BalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Someone applied to join the registry
        NewApplication(AccountId, Balance),
        /// Someone countered an application
        ApplicationCountered(AccountId, AccountId, Balance),
        /// A new vote for an application has been recorded
        VoteRecorded(AccountId, AccountId, Balance, bool),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// An application for this Origin is already pending
        ApplicationPending,
        /// A similar application is being challenged
        ApplicationChallenged,
        /// Not enough funds to pay the deposit
        NotEnoughFunds,
        /// The deposit is too small
        DepositTooSmall,
        /// The application linked to the member was not found
        ApplicationNotFound,
        /// The challenge linked ot the member was not found
        ChallengeNotFound,

        LockCreationOverflow,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as TcrModule {
        Applications get(applications): map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>>;
        Challenges get(challenges): map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>>;
        AmountLocked get(amount_locked): map hasher(blake2_256) T::AccountId => BalanceOf<T>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn apply(origin, metadata: Vec<u8>, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumApplicationAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(!<Applications<T>>::contains_key(sender.clone()), Error::<T>::ApplicationPending);
            ensure!(!<Challenges<T>>::contains_key(sender.clone()), Error::<T>::ApplicationChallenged);

            Self::lock_for(sender.clone(), deposit)?;

            <Applications<T>>::insert(sender.clone(), Application {
                candidate: sender.clone(),
                candidate_deposit: deposit,
                metadata: metadata,

                challenger: None,
                challenger_deposit: None,

                votes_for: None,
                votes_against: None,
            });

            Self::deposit_event(RawEvent::NewApplication(sender, deposit));
            Ok(())
        }

        /// Counter a pending application, this will initiate a challenge
        pub fn counter(origin, member: T::AccountId, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumChallengeAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(<Applications<T>>::contains_key(member.clone()), Error::<T>::ApplicationNotFound);

            Self::lock_for(sender.clone(), deposit)?;

            let mut application = <Applications<T>>::take(member.clone());
            application.challenger = Some(sender.clone());
            application.challenger_deposit = Some(deposit);

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::ApplicationCountered(member, sender, deposit));
            Ok(())
        }

        /// Vote in support or opposition of a given challenge
        pub fn vote(origin, member: T::AccountId, supporting: bool, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<Challenges<T>>::contains_key(member.clone()), Error::<T>::ChallengeNotFound);

            Self::lock_for(sender.clone(), deposit)?;

            let mut application = <Challenges<T>>::take(member.clone());

            if supporting {
                application.votes_for = Some(Self::helper_vote_increment(application.votes_for, deposit)?);
            } else {
                application.votes_against = Some(Self::helper_vote_increment(application.votes_against, deposit)?);
            }

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::VoteRecorded(member, sender, deposit, supporting));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Do not just call `set_lock`, rather increase the locked amount
    fn lock_for(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        if T::Currency::free_balance(&who) < amount {
            Err(Error::<T>::NotEnoughFunds)?;
        }

        let to_lock = <AmountLocked<T>>::get(who.clone())
            .checked_add(&amount)
            .ok_or(Error::<T>::LockCreationOverflow)?;

        T::Currency::set_lock(TCR_LOCK_ID, &who, to_lock, WithdrawReasons::all());
        <AmountLocked<T>>::insert(who, to_lock);

        Ok(())
    }

    /// Number of tokens supporting a given application
    fn get_supporting(who: T::AccountId) -> BalanceOf<T> {
        let application = <Challenges<T>>::get(who);
        application.candidate_deposit + application.votes_for.unwrap_or(0.into())
    }

    /// Number of tokens opposing a given application
    fn get_opposing(who: T::AccountId) -> BalanceOf<T> {
        let application = <Challenges<T>>::get(who);
        application.challenger_deposit.unwrap_or(0.into())
            + application.votes_against.unwrap_or(0.into())
    }

    fn helper_vote_increment(
        src_votes: Option<BalanceOf<T>>,
        add_votes: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let votes = match src_votes {
            Some(votes) => votes,
            None => 0.into(),
        };
        match votes.checked_add(&add_votes) {
            Some(votes) => Ok(votes),
            None => Err(DispatchError::Other("votes overflow")),
        }
    }
}
