#![cfg_attr(not(feature = "std"), no_std)]

//! This module implements a Root Of Trust linked to a `membership` or `tcr` pallet which
//! can be used to let entities represented by their `AccountId` manage certificates
//! and off-chain certificates in Public Key Infrastructure fashion (SSL / TLS like).

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{result::Result, DispatchError, DispatchResult},
    ensure,
    traits::{ChangeMembers, Currency, Get, Imbalance},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{traits::CheckedAdd, Perbill};
use sp_std::prelude::Vec;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct RootCertificate<AccountId, BlockNumber> {
    owner: AccountId,
    key: AccountId,
    creation: BlockNumber,
    renewed: BlockNumber,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency used to represent the voting power
    type Currency: Currency<Self::AccountId>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        Placeholder(AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        Placeholder,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RootOfTrustModule {
        Members get(members): Vec<T::AccountId>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {
    fn is_member(who: &T::AccountId) -> bool {
        Self::members().contains(who)
    }
}

impl<T: Trait> ChangeMembers<T::AccountId> for Module<T> {
    fn change_members_sorted(
        _incoming: &[T::AccountId],
        _outgoing: &[T::AccountId],
        new: &[T::AccountId],
    ) {
        <Members<T>>::put(new);
    }
}
