#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Config {
    pub token: Address,
    pub admin: Address,
}

#[contracttype]
pub enum DataKey {
    Config,
    NextStreamId,
    Stream(u64),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Active = 0,
    Paused = 1,
    Completed = 2,
    Cancelled = 3,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Stream {
    pub stream_id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub deposit_amount: i128,
    pub rate_per_second: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub withdrawn_amount: i128,
    pub status: StreamStatus,
}

#[contract]
pub struct FluxoraStream;

#[contractimpl]
impl FluxoraStream {
    /// Initialize the stream contract (e.g. set token and admin).
    pub fn init(env: Env, token: Address, admin: Address) -> () {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }
        let config = Config { token, admin };
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::NextStreamId, &1u64);
    }

    /// Create a new stream. Treasury locks USDC and sets rate, duration, cliff.
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        deposit_amount: i128,
        rate_per_second: i128,
        start_time: u64,
        cliff_time: u64,
        end_time: u64,
    ) -> u64 {
        sender.require_auth();

        let stream_id: u64 = env.storage().instance().get(&DataKey::NextStreamId).unwrap_or(1);
        
        let stream = Stream {
            stream_id,
            sender,
            recipient,
            deposit_amount,
            rate_per_second,
            start_time,
            cliff_time,
            end_time,
            withdrawn_amount: 0,
            status: StreamStatus::Active,
        };

        let key = DataKey::Stream(stream_id);
        env.storage().persistent().set(&key, &stream);
        env.storage().persistent().extend_ttl(&key, 17280, 120960);
        
        env.storage().instance().set(&DataKey::NextStreamId, &(stream_id + 1));
        
        stream_id
    }

    /// Pause an active stream (callable by sender/admin).
    pub fn pause_stream(_env: Env, _stream_id: u64) -> () {
        ()
    }

    /// Resume a paused stream.
    pub fn resume_stream(_env: Env, _stream_id: u64) -> () {
        ()
    }

    /// Cancel stream (callable by sender/admin). Unstreamed amount returns to sender.
    pub fn cancel_stream(_env: Env, _stream_id: u64) -> () {
        ()
    }

    /// Recipient withdraws accrued USDC up to (accrued - withdrawn).
    pub fn withdraw(_env: Env, _stream_id: u64) -> i128 {
        0
    }

    /// Calculate accrued amount: min((now - start_time) * rate_per_second, deposit_amount).
    /// With cliff: if now < cliff_time then 0.
    pub fn calculate_accrued(_env: Env, _stream_id: u64) -> i128 {
        0
    }

    /// Return current stream state for a given stream_id.
    pub fn get_stream_state(env: Env, stream_id: u64) -> Stream {
        env.storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .expect("Stream not found")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_storage_logic() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FluxoraStream);
        let client = FluxoraStreamClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.init(&token, &admin);

        env.mock_all_auths();
        let id = client.create_stream(&sender, &recipient, &1000, &1, &100, &110, &200);
        assert_eq!(id, 1);

        let stream = client.get_stream_state(&1);
        assert_eq!(stream.sender, sender);
        assert_eq!(stream.deposit_amount, 1000);
    }
}