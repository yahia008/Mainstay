#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contracterror, panic_with_error, symbol_short, Env, String, Symbol};

const METADATA_MAX_LEN: u32 = 256;

#[contracttype]
#[derive(Clone)]
pub struct Asset {
    pub asset_id: u64,
    pub asset_type: Symbol,
    pub metadata: String,
    pub owner: soroban_sdk::Address,
    pub registered_at: u64,
}

const ASSET_COUNT: Symbol = symbol_short!("A_COUNT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MetadataTooLong = 1,
}

fn asset_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("ASSET"), id)
}

#[contract]
pub struct AssetRegistry;

#[contractimpl]
impl AssetRegistry {
    pub fn register_asset(
        env: Env,
        asset_type: Symbol,
        metadata: String,
        owner: soroban_sdk::Address,
    ) -> u64 {
        owner.require_auth();
        if metadata.len() > METADATA_MAX_LEN {
            panic_with_error!(&env, Error::MetadataTooLong);
        }
        let id: u64 = env.storage().instance().get(&ASSET_COUNT).unwrap_or(0) + 1;
        let asset = Asset {
            asset_id: id,
            asset_type,
            metadata,
            owner,
            registered_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&asset_key(id), &asset);
        env.storage().instance().set(&ASSET_COUNT, &id);
        id
    }

    pub fn get_asset(env: Env, asset_id: u64) -> Asset {
        env.storage()
            .persistent()
            .get(&asset_key(asset_id))
            .expect("asset not found")
    }

    pub fn asset_count(env: Env) -> u64 {
        env.storage().instance().get(&ASSET_COUNT).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, Env, String};

    #[test]
    fn test_register_and_get_asset() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        let owner = soroban_sdk::Address::generate(&env);
        let id = client.register_asset(
            &symbol_short!("GENSET"),
            &String::from_str(&env, "Caterpillar 3516 Generator"),
            &owner,
        );
        assert_eq!(id, 1);

        let asset = client.get_asset(&id);
        assert_eq!(asset.asset_id, 1);
        assert_eq!(asset.owner, owner);
    }

    #[test]
    #[should_panic]
    fn test_register_asset_metadata_too_long() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        let owner = soroban_sdk::Address::generate(&env);
        // 257 'a' characters — one over the 256-byte limit
        let oversized = String::from_str(&env, &"a".repeat(257));
        client.register_asset(&symbol_short!("GENSET"), &oversized, &owner);
    }
}
