#![no_std]
//! PiGold registry and route-validation contract.
//! Coordinates metadata for independently accounted pair pools. It does not
//! custody tokens, aggregate balances, execute swaps, mint assets, or assert liquidity.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Vec,
};

const MAX_ROUTE_HOPS: u32 = 8;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAssetPair = 4,
    PoolAlreadyRegistered = 5,
    PoolNotFound = 6,
    PoolDisabled = 7,
    ReserveAlreadyAssigned = 8,
    InvalidRouteLength = 9,
    DuplicatePoolInRoute = 10,
    DuplicateReserveInRoute = 11,
    DisconnectedRoute = 12,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolRecord {
    pub pool_id: BytesN<32>,
    pub asset_a: String,
    pub asset_b: String,
    pub reserve_accounting_id: BytesN<32>,
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Pool(BytesN<32>),
    Reserve(BytesN<32>),
}

#[contract]
pub struct PiGoldCoordinator;

#[contractimpl]
impl PiGoldCoordinator {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Metadata registration only; it does not prove deployment or liquidity.
    pub fn register_pool(
        env: Env,
        admin: Address,
        pool_id: BytesN<32>,
        asset_a: String,
        asset_b: String,
        reserve_accounting_id: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if asset_a.len() == 0 || asset_b.len() == 0 || asset_a == asset_b {
            return Err(Error::InvalidAssetPair);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Pool(pool_id.clone()))
        {
            return Err(Error::PoolAlreadyRegistered);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Reserve(reserve_accounting_id.clone()))
        {
            return Err(Error::ReserveAlreadyAssigned);
        }
        let record = PoolRecord {
            pool_id: pool_id.clone(),
            asset_a,
            asset_b,
            reserve_accounting_id: reserve_accounting_id.clone(),
            enabled: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Pool(pool_id.clone()), &record);
        env.storage()
            .persistent()
            .set(&DataKey::Reserve(reserve_accounting_id), &pool_id);
        env.events().publish(
            (symbol_short!("pigold"), symbol_short!("pool_add")),
            pool_id,
        );
        Ok(())
    }

    /// Excludes a pool from new route plans; it does not freeze or move assets.
    pub fn set_pool_enabled(
        env: Env,
        admin: Address,
        pool_id: BytesN<32>,
        enabled: bool,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        let mut record = Self::load_pool(&env, &pool_id)?;
        record.enabled = enabled;
        env.storage()
            .persistent()
            .set(&DataKey::Pool(pool_id.clone()), &record);
        env.events().publish(
            (symbol_short!("pigold"), symbol_short!("pool_st")),
            (pool_id, enabled),
        );
        Ok(())
    }

    /// Resolves a directed path through enabled pair pools and returns its output asset.
    /// Every pool and reserve namespace may occur at most once in a route.
    pub fn validate_route(env: Env, route: Vec<BytesN<32>>) -> Result<String, Error> {
        let count = route.len();
        if count == 0 || count > MAX_ROUTE_HOPS {
            return Err(Error::InvalidRouteLength);
        }

        let mut seen_pools: Vec<BytesN<32>> = Vec::new(&env);
        let mut seen_reserves: Vec<BytesN<32>> = Vec::new(&env);
        let mut current_asset: Option<String> = None;

        for index in 0..count {
            let pool_id = route.get(index).ok_or(Error::PoolNotFound)?;
            if seen_pools.contains(pool_id.clone()) {
                return Err(Error::DuplicatePoolInRoute);
            }
            seen_pools.push_back(pool_id.clone());

            let record = Self::load_pool(&env, &pool_id)?;
            if !record.enabled {
                return Err(Error::PoolDisabled);
            }
            if seen_reserves.contains(record.reserve_accounting_id.clone()) {
                return Err(Error::DuplicateReserveInRoute);
            }
            seen_reserves.push_back(record.reserve_accounting_id.clone());

            current_asset = match current_asset {
                None => Some(record.asset_b.clone()),
                Some(previous) if previous == record.asset_a => Some(record.asset_b.clone()),
                Some(previous) if previous == record.asset_b => Some(record.asset_a.clone()),
                Some(_) => return Err(Error::DisconnectedRoute),
            };
        }
        current_asset.ok_or(Error::InvalidRouteLength)
    }

    pub fn get_pool(env: Env, pool_id: BytesN<32>) -> Result<PoolRecord, Error> {
        Self::load_pool(&env, &pool_id)
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if stored != admin.clone() {
            return Err(Error::Unauthorized);
        }
        admin.require_auth();
        Ok(())
    }

    fn load_pool(env: &Env, pool_id: &BytesN<32>) -> Result<PoolRecord, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Pool(pool_id.clone()))
            .ok_or(Error::PoolNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup(env: &Env) -> (Address, Address) {
        let contract_id = env.register(PiGoldCoordinator, ());
        let admin = Address::generate(env);
        env.mock_all_auths();
        PiGoldCoordinatorClient::new(env, &contract_id).initialize(&admin);
        (contract_id, admin)
    }

    #[test]
    fn validates_directed_multi_pool_route_without_combining_reserves() {
        let env = Env::default();
        let (id, admin) = setup(&env);
        let client = PiGoldCoordinatorClient::new(&env, &id);
        let pool_ab = BytesN::from_array(&env, &[1u8; 32]);
        let pool_bc = BytesN::from_array(&env, &[2u8; 32]);
        client.register_pool(
            &admin,
            &pool_ab,
            &String::from_str(&env, "PiCash"),
            &String::from_str(&env, "PiBank"),
            &BytesN::from_array(&env, &[11u8; 32]),
        );
        client.register_pool(
            &admin,
            &pool_bc,
            &String::from_str(&env, "PiBank"),
            &String::from_str(&env, "PiYellow"),
            &BytesN::from_array(&env, &[12u8; 32]),
        );
        let route = Vec::from_array(&env, [pool_ab, pool_bc]);
        assert_eq!(
            client.validate_route(&route),
            String::from_str(&env, "PiYellow")
        );
    }

    #[test]
    fn rejects_shared_reserve_accounting_identifier() {
        let env = Env::default();
        let (id, admin) = setup(&env);
        let client = PiGoldCoordinatorClient::new(&env, &id);
        let reserve = BytesN::from_array(&env, &[9u8; 32]);
        client.register_pool(
            &admin,
            &BytesN::from_array(&env, &[1u8; 32]),
            &String::from_str(&env, "PiCash"),
            &String::from_str(&env, "PiBank"),
            &reserve,
        );
        assert_eq!(
            client.try_register_pool(
                &admin,
                &BytesN::from_array(&env, &[2u8; 32]),
                &String::from_str(&env, "PiBank"),
                &String::from_str(&env, "PiYellow"),
                &reserve,
            ),
            Err(Ok(Error::ReserveAlreadyAssigned)),
        );
    }

    #[test]
    fn rejects_duplicate_pool_and_disconnected_route() {
        let env = Env::default();
        let (id, admin) = setup(&env);
        let client = PiGoldCoordinatorClient::new(&env, &id);
        let pool = BytesN::from_array(&env, &[3u8; 32]);
        client.register_pool(
            &admin,
            &pool,
            &String::from_str(&env, "PiCash"),
            &String::from_str(&env, "PiBank"),
            &BytesN::from_array(&env, &[13u8; 32]),
        );
        assert_eq!(
            client.try_validate_route(&Vec::from_array(&env, [pool.clone(), pool.clone()])),
            Err(Ok(Error::DuplicatePoolInRoute)),
        );
        let disconnected = BytesN::from_array(&env, &[4u8; 32]);
        client.register_pool(
            &admin,
            &disconnected,
            &String::from_str(&env, "PiYellow"),
            &String::from_str(&env, "PiPurple"),
            &BytesN::from_array(&env, &[14u8; 32]),
        );
        assert_eq!(
            client.try_validate_route(&Vec::from_array(&env, [pool, disconnected])),
            Err(Ok(Error::DisconnectedRoute)),
        );
    }

    #[test]
    fn disabled_pool_is_excluded_from_route_validation() {
        let env = Env::default();
        let (id, admin) = setup(&env);
        let client = PiGoldCoordinatorClient::new(&env, &id);
        let pool = BytesN::from_array(&env, &[5u8; 32]);
        client.register_pool(
            &admin,
            &pool,
            &String::from_str(&env, "PiCash"),
            &String::from_str(&env, "PiBank"),
            &BytesN::from_array(&env, &[15u8; 32]),
        );
        client.set_pool_enabled(&admin, &pool, &false);
        assert_eq!(
            client.try_validate_route(&Vec::from_array(&env, [pool])),
            Err(Ok(Error::PoolDisabled)),
        );
    }
}
