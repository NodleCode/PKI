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
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    traits::{CheckedAdd, MaybeDisplay, MaybeSerializeDeserialize, Member},
    Perbill,
};
use sp_std::{fmt::Debug, prelude::Vec};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct RootCertificate<AccountId, CertificateId, BlockNumber> {
    owner: AccountId,
    key: CertificateId,
    created: BlockNumber,
    renewed: BlockNumber,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency used to represent the voting power
    type Currency: Currency<Self::AccountId>;

    /// How a certificate public key is represented, typically `AccountId`
    type CertificateId: Member
        + Parameter
        + MaybeSerializeDeserialize
        + Debug
        + MaybeDisplay
        + Ord
        + Default;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        CertificateId = <T as Trait>::CertificateId,
    {
        /// A new slot has been booked
        SlotTaken(AccountId, CertificateId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// `origin` a member, this function may need a member account id
        NotAMember,
        /// Slot was already taken, you will need to use another certificate id
        SlotTaken,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RootOfTrustModule {
        Members get(members): Vec<T::AccountId>;
        Slots get(slots): map hasher(blake2_256) T::CertificateId => RootCertificate<T::AccountId, T::CertificateId, T::BlockNumber>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Book a certificate slot
        fn book_slot(origin, certificate_id: T::CertificateId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_member(&sender), Error::<T>::NotAMember);
            ensure!(!<Slots<T>>::contains_key(&certificate_id), Error::<T>::SlotTaken);

            let now = <system::Module<T>>::block_number();
            <Slots<T>>::insert(&certificate_id, RootCertificate {
                owner: sender.clone(),
                key: certificate_id.clone(),
                created: now,
                renewed: now,
            });

            Self::deposit_event(RawEvent::SlotTaken(sender, certificate_id));
            Ok(())
        }
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
