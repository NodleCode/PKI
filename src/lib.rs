#![cfg_attr(not(feature = "std"), no_std)]

//! This module implements a Token Curated Registry where members (represented by their
//! `AccountId`) are accepted based on the number of tokens staked in support to their
//! application.

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
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

            if T::Currency::free_balance(&sender) < deposit {
                Err(Error::<T>::NotEnoughFunds)?;
            }

            Self::lock_for(sender.clone(), deposit)?;

            <Applications<T>>::insert(sender.clone(), Application {
                candidate: sender.clone(),
                candidate_deposit: deposit,
                metadata: metadata,

                challenger: None,
                challenger_deposit: None,
            });

            Self::deposit_event(RawEvent::NewApplication(sender, deposit));
            Ok(())
        }

        /// Counter a pending application, this will initiate a challenge
        pub fn counter(origin, member: T::AccountId, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(deposit >= T::MinimumChallengeAmount::get(), Error::<T>::DepositTooSmall);
            ensure!(<Applications<T>>::contains_key(member.clone()), Error::<T>::ApplicationNotFound);

            if T::Currency::free_balance(&sender) < deposit {
                Err(Error::<T>::NotEnoughFunds)?;
            }

            Self::lock_for(sender.clone(), deposit)?;

            let mut application = <Applications<T>>::take(member.clone());
            application.challenger = Some(sender.clone());
            application.challenger_deposit = Some(deposit);

            <Challenges<T>>::insert(member.clone(), application);

            Self::deposit_event(RawEvent::ApplicationCountered(member, sender, deposit));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Do not just call `set_lock`, rather increase the locked amount
    fn lock_for(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let to_lock = <AmountLocked<T>>::get(who.clone())
            .checked_add(&amount)
            .ok_or(Error::<T>::LockCreationOverflow)?;

        T::Currency::set_lock(TCR_LOCK_ID, &who, to_lock, WithdrawReasons::all());
        <AmountLocked<T>>::insert(who, to_lock);

        Ok(())
    }
}
