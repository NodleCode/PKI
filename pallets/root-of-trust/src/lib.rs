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
    traits::{
        ChangeMembers, Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced,
        WithdrawReasons,
    },
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
    revoked: bool,
    validity: BlockNumber,
    child_revocations: Vec<CertificateId>,
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
    /// How much a new root certificate costs
    type SlotBookingCost: Get<BalanceOf<Self>>;
    /// How much renewing a root certificate costs
    type SlotRenewingCost: Get<BalanceOf<Self>>;
    /// How long a certificate is considered valid
    type SlotValidity: Get<Self::BlockNumber>;
    /// The module receiving funds paid by depositors, typically a company
    /// reserve
    type FundsCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        CertificateId = <T as Trait>::CertificateId,
    {
        /// A new slot has been booked
        SlotTaken(AccountId, CertificateId),
        /// An exisitng slot has been renewed (its validity period was extended)
        SlotRenewed(AccountId, CertificateId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// `origin` a member, this function may need a member account id
        NotAMember,
        /// Slot was already taken, you will need to use another certificate id
        SlotTaken,
        /// Not enough funds to pay the fee
        NotEnoughFunds,
        /// Slot is no longer valid
        NoLongerValid,
        /// `origin` is not the slot owner
        NotTheOwner,
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

            match T::Currency::withdraw(&sender, T::SlotBookingCost::get(), WithdrawReasons::all(), ExistenceRequirement::AllowDeath) {
                Ok(imbalance) => T::FundsCollector::on_unbalanced(imbalance),
                Err(_) => Err(Error::<T>::NotEnoughFunds)?,
            };

            let now = <system::Module<T>>::block_number();
            <Slots<T>>::insert(&certificate_id, RootCertificate {
                owner: sender.clone(),
                key: certificate_id.clone(),
                created: now,
                renewed: now,
                revoked: false,
                validity: T::SlotValidity::get(),
                child_revocations: vec![],
            });

            Self::deposit_event(RawEvent::SlotTaken(sender, certificate_id));
            Ok(())
        }

        fn renew_slot(origin, certificate: T::CertificateId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let mut slot = <Slots<T>>::get(&certificate);
            ensure!(Self::is_slot_valid(&slot), Error::<T>::NoLongerValid);
            ensure!(slot.owner == sender, Error::<T>::NotTheOwner);

            match T::Currency::withdraw(&sender, T::SlotRenewingCost::get(), WithdrawReasons::all(), ExistenceRequirement::AllowDeath) {
                Ok(imbalance) => T::FundsCollector::on_unbalanced(imbalance),
                Err(_) => Err(Error::<T>::NotEnoughFunds)?,
            };

            slot.renewed = <system::Module<T>>::block_number();

            <Slots<T>>::insert(&certificate, slot);

            Self::deposit_event(RawEvent::SlotRenewed(sender, certificate));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn is_member(who: &T::AccountId) -> bool {
        Self::members().contains(who)
    }

    fn is_slot_valid(
        slot: &RootCertificate<T::AccountId, T::CertificateId, T::BlockNumber>,
    ) -> bool {
        let owner_is_member = Self::is_member(&slot.owner);
        let revoked = slot.revoked;
        let expired = slot.renewed + slot.validity <= <system::Module<T>>::block_number();

        owner_is_member && !revoked && !expired
    }

    fn is_root_certificate_valid(cert: &T::CertificateId) -> bool {
        let exists = <Slots<T>>::contains_key(cert);
        let slot = <Slots<T>>::get(cert);

        exists && Self::is_slot_valid(&slot)
    }

    fn is_child_certificate_valid(root: &T::CertificateId, child: &T::CertificateId) -> bool {
        let equals = root == child;
        let root_valid = Self::is_root_certificate_valid(root);
        let revoked = <Slots<T>>::get(root).child_revocations.contains(child);

        // TODO: let's support signature verification here

        !equals && root_valid && !revoked
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
