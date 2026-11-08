#![no_std]
//! Generic, permissioned PiRC test asset token.
//! Test-only: this contract does not represent Pi, USD backing, a peg, reserves,
//! redemption, or an official Pi Network asset.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

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
    InvalidDecimals = 11,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Name,
    Symbol,
    Decimals,
    MaxSupply,
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
pub struct PiRcTestAsset;

impl PiRcTestAsset {
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

    fn total_supply_of(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    fn move_balance(
        env: &Env,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let from_balance = Self::balance_of(env, from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let to_balance = Self::balance_of(env, to);
        let next_to = to_balance.checked_add(amount).ok_or(Error::ArithmeticError)?;
        let next_from = from_balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        Self::set_balance(env, from, next_from);
        Self::set_balance(env, to, next_to);
        env.events().publish(
            (symbol_short!("asset"), symbol_short!("transfer")),
            (from.clone(), to.clone(), amount),
        );
        Ok(())
    }
}

#[contractimpl]
impl PiRcTestAsset {
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        max_supply: i128,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        if decimals > 18 {
            return Err(Error::InvalidDecimals);
        }
        if max_supply <= 0 {
            return Err(Error::InvalidAmount);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::MaxSupply, &max_supply);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.events().publish(
            (symbol_short!("asset"), symbol_short!("init")),
            (admin, name, symbol, decimals, max_supply),
        );
        Ok(())
    }

    pub fn mint(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let supply = Self::total_supply_of(&env);
        let cap: i128 = env.storage().instance().get(&DataKey::MaxSupply).ok_or(Error::NotInitialized)?;
        let next_supply = supply.checked_add(amount).ok_or(Error::ArithmeticError)?;
        if next_supply > cap {
            return Err(Error::SupplyCapExceeded);
        }
        let balance = Self::balance_of(&env, &to);
        let next_balance = balance.checked_add(amount).ok_or(Error::ArithmeticError)?;
        env.storage().instance().set(&DataKey::TotalSupply, &next_supply);
        Self::set_balance(&env, &to, next_balance);
        env.events().publish(
            (symbol_short!("asset"), symbol_short!("mint")),
            (to, amount, next_supply),
        );
        Ok(())
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
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
            (symbol_short!("asset"), symbol_short!("approve")),
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
        let supply = Self::total_supply_of(&env);
        let next_balance = balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_supply = supply.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        Self::set_balance(&env, &from, next_balance);
        env.storage().instance().set(&DataKey::TotalSupply, &next_supply);
        env.events().publish(
            (symbol_short!("asset"), symbol_short!("burn")),
            (from, amount, next_supply),
        );
        Ok(())
    }

    pub fn burn_from(
        env: Env,
        spender: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();
        Self::require_initialized(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let key = DataKey::Allowance(from.clone(), spender.clone());
        let mut allowance: AllowanceData = env.storage().persistent().get(&key).ok_or(Error::InsufficientAllowance)?;
        if allowance.expiration_ledger < env.ledger().sequence() {
            return Err(Error::AllowanceExpired);
        }
        if allowance.amount < amount {
            return Err(Error::InsufficientAllowance);
        }
        let balance = Self::balance_of(&env, &from);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let supply = Self::total_supply_of(&env);
        allowance.amount = allowance.amount.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_balance = balance.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        let next_supply = supply.checked_sub(amount).ok_or(Error::ArithmeticError)?;
        env.storage().persistent().set(&key, &allowance);
        Self::set_balance(&env, &from, next_balance);
        env.storage().instance().set(&DataKey::TotalSupply, &next_supply);
        env.events().publish(
            (symbol_short!("asset"), symbol_short!("burn")),
            (from, amount, next_supply),
        );
        Ok(())
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        Self::balance_of(&env, &id)
    }

    pub fn total_supply(env: Env) -> i128 {
        Self::total_supply_of(&env)
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Decimals).unwrap_or(0)
    }

    pub fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap_or(String::from_str(&env, "Uninitialized"))
    }

    pub fn symbol(env: Env) -> String {
        env.storage().instance().get(&DataKey::Symbol).unwrap_or(String::from_str(&env, "UNINIT"))
    }

    pub fn admin(env: Env) -> Result<Address, Error> {
        Self::require_initialized(&env)
    }

    pub fn max_supply(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::MaxSupply).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::{Error, PiRcTestAsset, PiRcTestAssetClient};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env, String,
    };

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(PiRcTestAsset, ());
        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let client = PiRcTestAssetClient::new(&env, &id);
        client.initialize(
            &admin,
            &String::from_str(&env, "PiRC Test Asset"),
            &String::from_str(&env, "PIT"),
            &7,
            &1_000_000_000,
        );
        (env, id, admin, alice, bob)
    }

    #[test]
    fn metadata_mint_transfer_and_burn_work() {
        let (env, id, admin, alice, bob) = setup();
        let client = PiRcTestAssetClient::new(&env, &id);
        assert_eq!(client.name(), String::from_str(&env, "PiRC Test Asset"));
        assert_eq!(client.symbol(), String::from_str(&env, "PIT"));
        assert_eq!(client.decimals(), 7);

        client.mint(&admin, &alice, &1_000);
        client.transfer(&alice, &bob, &300);
        client.burn(&bob, &100);

        assert_eq!(client.balance(&alice), 700);
        assert_eq!(client.balance(&bob), 200);
        assert_eq!(client.total_supply(), 900);
    }

    #[test]
    fn allowance_is_consumed_and_expires() {
        let (env, id, admin, alice, bob) = setup();
        let client = PiRcTestAssetClient::new(&env, &id);
        let spender = Address::generate(&env);
        client.mint(&admin, &alice, &500);
        client.approve(&alice, &spender, &250, &(env.ledger().sequence() + 10));
        client.transfer_from(&spender, &alice, &bob, &100);
        assert_eq!(client.allowance(&alice, &spender), 150);
        client.burn_from(&spender, &alice, &25);
        assert_eq!(client.allowance(&alice, &spender), 125);
        assert_eq!(client.balance(&alice), 375);
        // transfer_from moves tokens; only burn_from reduces total supply.
        assert_eq!(client.total_supply(), 475);

        env.ledger().set_sequence_number(env.ledger().sequence() + 11);
        assert_eq!(client.allowance(&alice, &spender), 0);
        assert!(matches!(
            client.try_transfer_from(&spender, &alice, &bob, &1),
            Err(Ok(Error::AllowanceExpired))
        ));
    }

    #[test]
    fn unauthorized_mint_and_supply_cap_are_rejected() {
        let (env, id, admin, alice, _bob) = setup();
        let client = PiRcTestAssetClient::new(&env, &id);
        let attacker = Address::generate(&env);
        assert!(matches!(
            client.try_mint(&attacker, &alice, &100),
            Err(Ok(Error::Unauthorized))
        ));
        client.mint(&admin, &alice, &1_000_000_000);
        assert!(matches!(
            client.try_mint(&admin, &alice, &1),
            Err(Ok(Error::SupplyCapExceeded))
        ));
    }

    #[test]
    fn invalid_decimals_and_reinitialization_are_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(PiRcTestAsset, ());
        let client = PiRcTestAssetClient::new(&env, &id);
        let admin = Address::generate(&env);
        let name = String::from_str(&env, "Invalid");
        let symbol = String::from_str(&env, "INV");
        assert!(matches!(
            client.try_initialize(&admin, &name, &symbol, &19, &100),
            Err(Ok(Error::InvalidDecimals))
        ));
        client.initialize(&admin, &name, &symbol, &7, &100);
        assert!(matches!(
            client.try_initialize(&admin, &name, &symbol, &7, &100),
            Err(Ok(Error::AlreadyInitialized))
        ));
    }
}


#[cfg(test)]
mod integration_tests;
