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
    traits::{ChangeMembers, Currency, Get, LockIdentifier, LockableCurrency},
    Parameter,
};
use sp_runtime::{
    traits::{Dispatchable, EnsureOrigin},
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
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency used to represent the voting power
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    /// Minimum amount of tokens required to apply
    type MinimumApplicationAmount: Get<BalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Someone applied to join the registry
        NewApplication(AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// An application for this Origin is already pending
        ApplicationPending,
        /// Not enough funds to pay the deposit
        NotEnoughFunds,
        /// The application deposit is too small
        DepositTooSmall,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as TcrModule {
        Applications get(applications): map hasher(blake2_256) T::AccountId => Application<T::AccountId, BalanceOf<T>>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn apply(origin, metadata: Vec<u8>, deposit: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            <Applications<T>>::insert(sender.clone(), Application {
                candidate: sender.clone(),
                candidate_deposit: deposit,
                metadata: metadata,
            });

            Self::deposit_event(RawEvent::NewApplication(sender, deposit));
            Ok(())
        }
    }
}
