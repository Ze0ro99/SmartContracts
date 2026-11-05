#![no_std]
//! OUSD Test Token — permissioned, test-only Soroban token reference.
//! This contract does not represent USD backing, a peg, reserves or redemption.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

const DECIMALS: u32 = 7;
const MAX_SUPPLY: i128 = 10_000_000_000_000_000; // 1,000,000,000 tokens at 7 decimals

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
    InsufficientAllowance = 6,
    AllowanceExpired = 7,
    SupplyCapExceeded = 8,
    ArithmeticError = 9,
    InvalidExpiration = 10,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    TotalSupply,
    Balance(Address),
    Allowance(Address, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceData {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contract]
pub struct OusdTestToken;

impl OusdTestToken {
    fn require_initialized(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        admin.require_auth();
        let stored = Self::require_initialized(env)?;
        if stored != admin.clone() {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn balance_of(env: &Env, owner: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner.clone()))
            .unwrap_or(0)
    }

    fn set_balance(env: &Env, owner: &Address, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &amount);
    }

    fn read_total_supply(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    fn set_total_supply(env: &Env, amount: i128) {
        env.storage().instance().set(&DataKey::TotalSupply, &amount);
    }

    fn move_balance(env: &Env, from: &Address, to: &Address, amount: i128) -> Result<(), Error> {
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }
        let from_balance = Self::balance_of(env, from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        if from == to || amount == 0 {
            return Ok(());
        }
        let to_balance = Self::balance_of(env, to);
        let next_to = to_balance.checked_add(amount).ok_or(Error::ArithmeticError)?;
        let next_from = from_balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        Self::set_balance(env, from, next_from);
        Self::set_balance(env, to, next_to);
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("transfer")),
            (from.clone(), to.clone(), amount),
        );
        Ok(())
    }
}

#[contractimpl]
impl OusdTestToken {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Self::set_total_supply(&env, 0);
        env.events().publish((symbol_short!("ousd"), symbol_short!("init")), admin);
        Ok(())
    }

    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let supply = Self::read_total_supply(&env);
        let next_supply = supply.checked_add(amount).ok_or(Error::ArithmeticError)?;
        if next_supply > MAX_SUPPLY {
            return Err(Error::SupplyCapExceeded);
        }
        let balance = Self::balance_of(&env, &to);
        let next_balance = balance.checked_add(amount).ok_or(Error::ArithmeticError)?;
        Self::set_total_supply(&env, next_supply);
        Self::set_balance(&env, &to, next_balance);
        env.events().publish((symbol_short!("ousd"), symbol_short!("mint")), (to, amount));
        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        Self::require_initialized(&env)?;
        Self::move_balance(&env, &from, &to, amount)
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), Error> {
        from.require_auth();
        Self::require_initialized(&env)?;
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }
        if expiration_ledger <= env.ledger().sequence() {
            return Err(Error::InvalidExpiration);
        }
        let value = AllowanceData { amount, expiration_ledger };
        env.storage().persistent().set(
            &DataKey::Allowance(from.clone(), spender.clone()),
            &value,
        );
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("approve")),
            (from, spender, amount, expiration_ledger),
        );
        Ok(())
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let value: Option<AllowanceData> = env.storage().persistent().get(
            &DataKey::Allowance(from, spender),
        );
        match value {
            Some(item) if item.expiration_ledger >= env.ledger().sequence() => item.amount,
            _ => 0,
        }
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();
        Self::require_initialized(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let key = DataKey::Allowance(from.clone(), spender.clone());
        let mut value: AllowanceData = env.storage().persistent().get(&key).ok_or(Error::InsufficientAllowance)?;
        if value.expiration_ledger < env.ledger().sequence() {
            return Err(Error::AllowanceExpired);
        }
        if value.amount < amount {
            return Err(Error::InsufficientAllowance);
        }
        Self::move_balance(&env, &from, &to, amount)?;
        value.amount = value.amount.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        env.storage().persistent().set(&key, &value);
        Ok(())
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        Self::require_initialized(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let balance = Self::balance_of(&env, &from);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let supply = Self::read_total_supply(&env);
        let next_balance = balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_supply = supply.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        Self::set_balance(&env, &from, next_balance);
        Self::set_total_supply(&env, next_supply);
        env.events().publish((symbol_short!("ousd"), symbol_short!("burn")), (from, amount));
        Ok(())
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) -> Result<(), Error> {
        spender.require_auth();
        Self::require_initialized(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let key = DataKey::Allowance(from.clone(), spender.clone());
        let mut value: AllowanceData = env.storage().persistent().get(&key).ok_or(Error::InsufficientAllowance)?;
        if value.expiration_ledger < env.ledger().sequence() {
            return Err(Error::AllowanceExpired);
        }
        if value.amount < amount {
            return Err(Error::InsufficientAllowance);
        }
        let balance = Self::balance_of(&env, &from);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }
        value.amount = value.amount.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_balance = balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_supply = Self::read_total_supply(&env).checked_sub(amount).ok_or(Error::ArithmeticError)?;
        env.storage().persistent().set(&key, &value);
        Self::set_balance(&env, &from, next_balance);
        Self::set_total_supply(&env, next_supply);
        env.events().publish((symbol_short!("ousd"), symbol_short!("burn")), (from, amount));
        Ok(())
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        Self::balance_of(&env, &id)
    }

    pub fn total_supply(env: Env) -> i128 {
        Self::read_total_supply(&env)
    }

    pub fn decimals(_env: Env) -> u32 {
        DECIMALS
    }

    pub fn name(env: Env) -> String {
        String::from_str(&env, "OUSD Test Token")
    }

    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "OUSD")
    }

    pub fn admin(env: Env) -> Result<Address, Error> {
        Self::require_initialized(&env)
    }

    pub fn max_supply(_env: Env) -> i128 {
        MAX_SUPPLY
    }
}

#[cfg(test)]
mod test {
    use super::{OusdTestToken, OusdTestTokenClient, MAX_SUPPLY};
    use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(OusdTestToken, ());
        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let client = OusdTestTokenClient::new(&env, &id);
        client.initialize(&admin);
        (env, id, admin, alice, bob)
    }

    #[test]
    fn metadata_and_initial_supply_are_test_only_defaults() {
        let (env, id, _admin, _alice, _bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        assert_eq!(client.name(), String::from_str(&env, "OUSD Test Token"));
        assert_eq!(client.symbol(), String::from_str(&env, "OUSD"));
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.total_supply(), 0);
    }

    #[test]
    fn admin_mints_and_holders_transfer_and_burn() {
        let (env, id, admin, alice, bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        client.mint(&admin, &alice, &1_000);
        client.transfer(&alice, &bob, &300);
        assert_eq!(client.balance(&alice), 700);
        assert_eq!(client.balance(&bob), 300);
        client.burn(&bob, &100);
        assert_eq!(client.balance(&bob), 200);
        assert_eq!(client.total_supply(), 900);
    }

    #[test]
    fn allowance_transfer_from_consumes_approval() {
        let (env, id, admin, alice, bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        let spender = Address::generate(&env);
        client.mint(&admin, &alice, &500);
        client.approve(&alice, &spender, &250, &(env.ledger().sequence() + 20));
        client.transfer_from(&spender, &alice, &bob, &100);
        assert_eq!(client.allowance(&alice, &spender), 150);
        assert_eq!(client.balance(&bob), 100);
    }

    #[test]
    fn mint_above_cap_and_negative_transfer_are_rejected() {
        let (env, id, admin, alice, bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        assert!(client.try_mint(&admin, &alice, &(MAX_SUPPLY + 1)).is_err());
        client.mint(&admin, &alice, &100);
        assert!(client.try_transfer(&alice, &bob, &-1).is_err());
    }

    #[test]
    fn unauthorized_admin_cannot_mint() {
        let (env, id, _admin, alice, _bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        let stranger = Address::generate(&env);
        assert!(client.try_mint(&stranger, &alice, &1).is_err());
    }

    #[test]
    fn second_initialization_is_rejected() {
        let (env, id, admin, _alice, _bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        assert!(client.try_initialize(&admin).is_err());
    }

    #[test]
    fn negative_mint_and_zero_burn_are_rejected() {
        let (env, id, admin, alice, _bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        assert!(client.try_mint(&admin, &alice, &-1).is_err());
        assert!(client.try_burn(&alice, &0).is_err());
    }

    #[test]
    fn expired_allowance_is_not_spendable() {
        let (env, id, admin, alice, bob) = setup();
        let client = OusdTestTokenClient::new(&env, &id);
        let spender = Address::generate(&env);
        client.mint(&admin, &alice, &500).unwrap();
        let current = env.ledger().sequence();
        client.approve(&alice, &spender, &200, &(current + 1));
        env.ledger().set_sequence_number(current + 2);
        assert_eq!(client.allowance(&alice, &spender), 0);
        assert!(client.try_transfer_from(&spender, &alice, &bob, &100).is_err());
    }
}
