#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, String, Env, Vec};
use crate::datakey::DataKey;

#[contract]
pub struct StallionNFT;


#[contracttype]
pub enum DataKey {
    Owner(i128),
    TokenCount,
    Approvals(i128),
    Whitelist,
}


#[contractimpl]
impl StallionNFT {

    const SUPPLY: i128 = 2000;
    const NAME: &'static str = "StallionNFT";
    const SYMBOL: &'static str = "SNFT";
    const METADATA: &'static str = "https://ipfs.io/ipfs/bafkreibzw25uz3cxnpd4ditc2s7ngyea2hpq45s7psbs27dm3z6r57rzbe";
    const IMAGE: &'static str = "https://ipfs.io/ipfs/bafybeichocyvocmrrixgunzlrcnj4u7sbg3cst54mp3e3begu4qiphe3jq";

    /// Constructor that initializes the counter with a starting value
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&"counter", &counter);
    }

    pub fn name(env: Env) -> String {
        String::from_str(&env, Self::NAME)
    }

    pub fn symbol(env: Env) -> String {
        String::from_str(&env, Self::SYMBOL)
    }

    pub fn token_uri(env: Env) -> String {
        String::from_str(&env, Self::METADATA)
    }

    pub fn token_image(env: Env) -> String {
        String::from_str(&env, Self::IMAGE)
    }

    pub fn add_to_whitelist(env: Env, address: Address) {
        let mut whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist).unwrap_or_else(|| Vec::new(&env));
        if !whitelist.contains(&address) {
            whitelist.push_back(address.clone());
            env.storage().persistent().set(&DataKey::Whitelist, &whitelist);
        }
    }

    pub fn remove_from_whitelist(env: Env, admin: Address, address: Address) {
        admin.require_auth(); // Ensure only authorized users can remove from the whitelist
        let mut whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist).unwrap_or_else(|| Vec::new(&env));
        if let Some(pos) = whitelist.iter().position(|x| *x == address) {
            whitelist.remove(pos);
            env.storage().persistent().set(&DataKey::Whitelist, &whitelist);
        }
    }

    pub fn is_approved(env: Env, operator: Address, token_id: i128) -> bool {
        let key = DataKey::Approvals(token_id);
        let approvals = env.storage().persistent().get::<DataKey, Vec<Address>>(&key).unwrap_or_else(|| Vec::new(&env));
        approvals.contains(&operator)
    }

    pub fn transfer(env: Env, owner: Address, to: Address, token_id: i128) {
        owner.require_auth();
        let actual_owner = Self::owner_of(env.clone(), token_id);
        if owner == actual_owner {
            env.storage().persistent().set(&DataKey::Owner(token_id), &to);
            env.storage().persistent().remove(&DataKey::Approvals(token_id));
            env.events().publish((symbol_short!("Transfer"),), (owner, to, token_id));
        } else {
            panic!("Not the token owner");
        }
    }

    pub fn mint(env: Env, to: Address) {
        let whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist).unwrap_or_else(|| Vec::new(&env));
        assert!(whitelist.contains(&to), "Address not whitelisted");
        let mut token_count: i128 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);
        assert!(token_count < Self::SUPPLY, "Maximum token supply reached");
        token_count += 1;
        env.storage().persistent().set(&DataKey::TokenCount, &token_count);
        env.storage().persistent().set(&DataKey::Owner(token_count), &to);
        env.events().publish((symbol_short!("Mint"),), (to, token_count));
    }


    pub fn approve(env: Env, owner: Address, to: Address, token_id: i128) {
        owner.require_auth();
        let actual_owner = Self::owner_of(env.clone(), token_id);
        if owner == actual_owner {
            let key = DataKey::Approvals(token_id);
            let mut approvals = env.storage().persistent().get::<DataKey, Vec<Address>>(&key).unwrap_or_else(|| Vec::new(&env));
            if !approvals.contains(&to) {
                approvals.push_back(to.clone());
                env.storage().persistent().set(&key, &approvals);
                env.events().publish((symbol_short!("Approval"),), (owner, to, token_id));
            }
        } else {
            panic!("Not the token owner");
        }
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: i128) {
        spender.require_auth();
        let actual_owner = Self::owner_of(env.clone(), token_id);
        if from != actual_owner {
            panic!("From not owner");
        }
        let key = DataKey::Approvals(token_id);
        let approvals = env.storage().persistent().get::<DataKey, Vec<Address>>(&key).unwrap_or_else(|| Vec::new(&env));
        if !approvals.contains(&spender) {
            panic!("Spender is not approved for this token");
        }
        env.storage().persistent().set(&DataKey::Owner(token_id), &to);
        env.storage().persistent().remove(&DataKey::Approvals(token_id));
        env.events().publish((symbol_short!("Transfer"),), (from, to, token_id));
    }
}

mod test;
