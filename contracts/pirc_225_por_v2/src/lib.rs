#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    AuditorNotAuthorized = 4,
    InvalidDecimals = 5,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Auditor(Address),
    Attestation(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveAttestation {
    pub reserve_atomic: u128,
    pub liability_atomic: u128,
    pub decimals: u32,
    pub as_of: u64,
    pub report_hash: BytesN<32>,
    pub auditor: Address,
}

#[contract]
pub struct ProofOfReserves;

impl ProofOfReserves {
    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        admin.require_auth();
        let stored: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        match stored {
            Some(value) if value == *admin => Ok(()),
            Some(_) => Err(Error::Unauthorized),
            None => Err(Error::NotInitialized),
        }
    }
}

#[contractimpl]
impl ProofOfReserves {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) { return Err(Error::AlreadyInitialized); }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_auditor(env: Env, admin: Address, auditor: Address, enabled: bool) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Auditor(auditor.clone()), &enabled);
        env.events().publish((symbol_short!("por"), symbol_short!("auditor")), (auditor, enabled));
        Ok(())
    }

    pub fn record_attestation(
        env: Env,
        auditor: Address,
        asset_id: BytesN<32>,
        reserve_atomic: u128,
        liability_atomic: u128,
        decimals: u32,
        report_hash: BytesN<32>,
    ) -> Result<(), Error> {
        auditor.require_auth();
        let enabled: bool = env.storage().persistent().get(&DataKey::Auditor(auditor.clone())).unwrap_or(false);
        if !enabled { return Err(Error::AuditorNotAuthorized); }
        if decimals > 18 { return Err(Error::InvalidDecimals); }
        let record = ReserveAttestation {
            reserve_atomic,
            liability_atomic,
            decimals,
            as_of: env.ledger().timestamp(),
            report_hash,
            auditor: auditor.clone(),
        };
        env.storage().persistent().set(&DataKey::Attestation(asset_id.clone()), &record);
        env.events().publish((symbol_short!("por"), symbol_short!("attest")), (asset_id, reserve_atomic, liability_atomic, auditor));
        Ok(())
    }

    pub fn get_attestation(env: Env, asset_id: BytesN<32>) -> Option<ReserveAttestation> {
        env.storage().persistent().get(&DataKey::Attestation(asset_id))
    }

    pub fn is_covered(env: Env, asset_id: BytesN<32>) -> Option<bool> {
        let record: Option<ReserveAttestation> = env.storage().persistent().get(&DataKey::Attestation(asset_id));
        record.map(|r| r.reserve_atomic >= r.liability_atomic)
    }
}

#[cfg(test)]
mod test {
    use super::{ProofOfReserves, ProofOfReservesClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    #[test]
    fn only_authorized_auditor_can_record_and_coverage_uses_same_atomic_units() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(ProofOfReserves, ());
        let client = ProofOfReservesClient::new(&env, &id);
        let admin = Address::generate(&env);
        let auditor = Address::generate(&env);
        let asset = BytesN::from_array(&env, &[3u8; 32]);
        let report = BytesN::from_array(&env, &[4u8; 32]);
        client.initialize(&admin).unwrap();
        client.set_auditor(&admin, &auditor, &true).unwrap();
        client.record_attestation(&auditor, &asset, &1_000, &900, &7, &report).unwrap();
        assert_eq!(client.is_covered(&asset), Some(true));
    }
}
