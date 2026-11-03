#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn id(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    #[test]
    fn settles_only_after_exact_conservation() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let participant = Address::generate(&env);

        EconomicSettlement::initialize(env.clone(), admin.clone()).unwrap();
        EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

        let period_id = id(&env, 1);
        let distributable = EconomicSettlement::propose_period(
            env.clone(), operator.clone(), period_id.clone(), id(&env, 2),
            1_000, 200, 100, 100, 50, 50, 1, id(&env, 3),
        ).unwrap();
        assert_eq!(distributable, 500);

        EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
        assert_eq!(
            EconomicSettlement::settle_period(
                env.clone(), operator.clone(), period_id.clone(), id(&env, 4)
            ),
            Err(Error::AllocationNotConserved)
        );

        EconomicSettlement::record_allocation(
            env.clone(), operator.clone(), period_id.clone(), participant.clone(), 500, id(&env, 5)
        ).unwrap();
        EconomicSettlement::settle_period(
            env.clone(), operator.clone(), period_id.clone(), id(&env, 6)
        ).unwrap();

        let period = EconomicSettlement::get_period(env.clone(), period_id.clone()).unwrap();
        assert_eq!(period.status, PeriodStatus::Settled);
        assert_eq!(period.allocated, period.distributable);
        assert_eq!(
            EconomicSettlement::settle_period(env, operator, period_id, id(&Env::default(), 7)),
            Err(Error::AlreadySettled)
        );
    }

    #[test]
    fn rejects_underfunded_period() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        EconomicSettlement::initialize(env.clone(), admin).unwrap();
        EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

        let result = EconomicSettlement::propose_period(
            env.clone(), operator, id(&env, 8), id(&env, 9),
            100, 70, 20, 20, 0, 0, 1, id(&env, 10),
        );
        assert_eq!(result, Err(Error::InvalidDeductions));
    }
}
