use rstd::prelude::*;
use parity_codec::Codec;
use support::{dispatch::Result, StorageMap, Parameter, StorageValue, decl_storage, decl_module, decl_event, ensure};
use system::{self, ensure_signed};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn init(origin) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_init() == false, "Already initialized.");
            ensure!(Self::owner() == sender, "Only owner can initialize.");

            <BalanceOf<T>>::insert(sender.clone(), Self::total_supply());
            <Init<T>>::put(true);

            Ok(())
        }

        fn transfer(_origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(_origin)?;
            Self::_transfer(sender, to, value)
        }

        fn approve(origin, spender: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

            let allowance = Self::allowance((sender.clone(), spender.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;

            <Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(sender, spender, value));
            Ok(())
        }

        fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            ensure!(<Allowance<T>>::exists((from.clone(), to.clone())), "Allowance does not exists.");

            let allowance = Self::allowance((from.clone(), to.clone()));

            ensure!(allowance >= value, "Not enough allowance.");

            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;

            <Allowance<T>>::insert((from.clone(), to.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(from.clone(), to.clone(), value));
            Self::_transfer(from, to, value)
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Erc20 {
        Init get(is_init): bool;
        Owner get(owner) config(): T::AccountId;
        TotalSupply get(total_supply) config(): T::TokenBalance;
        Name get(name) config(): Vec<u8>;
        Ticker get(ticker) config(): Vec<u8>;
        BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
        Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;

    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Balance = <T as self::Trait>::TokenBalance {
        // from, to, value
        Transfer(AccountId, AccountId, Balance),
        // owner, spender, value
        Approval(AccountId, AccountId, Balance),
    }
);

impl<T: Trait> Module<T> {
    fn _transfer(from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
        ensure!(<BalanceOf<T>>::exists(from.clone()), "Account does not own this token");

        let sender_balance = Self::balance_of(from.clone());

        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of(to.clone());
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;

        <BalanceOf<T>>::insert(from.clone(), updated_from_balance);
        <BalanceOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(from, to, value));
        Ok(())
    }
}