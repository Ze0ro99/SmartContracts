#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Subscription(Address),
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            return;
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        if amount <= 0 {
            return;
        }
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        let Some(admin) = admin else {
            return;
        };
        admin.require_auth();

        let key = DataKey::Balance(to.clone());
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if let Some(updated) = balance.checked_add(amount) {
            env.storage().persistent().set(&key, &updated);
        }
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> bool {
        if amount <= 0 {
            return false;
        }
        from.require_auth();
        if from == to {
            return true;
        }

        let from_key = DataKey::Balance(from.clone());
        let to_key = DataKey::Balance(to.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
        if from_bal < amount {
            return false;
        }
        let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
        let Some(new_from) = from_bal.checked_sub(amount) else {
            return false;
        };
        let Some(new_to) = to_bal.checked_add(amount) else {
            return false;
        };

        env.storage().persistent().set(&from_key, &new_from);
        env.storage().persistent().set(&to_key, &new_to);
        true
    }

    pub fn subscribe(env: Env, user: Address, plan: String) -> bool {
        user.require_auth();
        env.storage().persistent().set(&DataKey::Subscription(user), &plan);
        true
    }

    pub fn balance_of(env: Env, address: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(address)).unwrap_or(0)
    }

    pub fn ping(env: Env) -> String {
        String::from_str(&env, "pirc-economy-ok")
    }
}

#[cfg(test)]
mod test {
    use super::{Contract, ContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn initialization_cannot_be_taken_over() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let replacement = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);
        client.initialize(&replacement);
        client.mint(&recipient, &100);
        assert_eq!(client.balance_of(&recipient), 100);
    }

    #[test]
    fn rejects_non_positive_mint_and_transfer_amounts() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        client.initialize(&admin);
        client.mint(&user, &-10);
        client.mint(&user, &0);
        assert_eq!(client.balance_of(&user), 0);
        assert!(!client.transfer(&user, &Address::generate(&env), &-1));
        assert!(!client.transfer(&user, &Address::generate(&env), &0));
    }

    #[test]
    fn self_transfer_does_not_create_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        client.initialize(&admin);
        client.mint(&user, &50);
        assert!(client.transfer(&user, &user, &50));
        assert_eq!(client.balance_of(&user), 50);
    }

    #[test]
    fn transfer_preserves_total_and_rejects_overflow() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        client.initialize(&admin);
        client.mint(&sender, &100);
        assert!(client.transfer(&sender, &receiver, &40));
        assert_eq!(client.balance_of(&sender), 60);
        assert_eq!(client.balance_of(&receiver), 40);
    }
}
