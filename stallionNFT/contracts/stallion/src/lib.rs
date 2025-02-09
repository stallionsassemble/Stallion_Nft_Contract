#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, String, Env, Vec, symbol_short, Bytes};

// Define the StallionNFT contract
#[contract]
pub struct StallionNFT;

// Define the keys used for storing data in the contract's storage
#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Owner(i128),          // Key for storing the owner of a token
    TokenCount,           // Key for storing the total number of tokens minted
    Approvals(i128),      // Key for storing approved addresses for a token
    Whitelist,            // Key for storing the whitelist of addresses allowed to mint
    Admin(Address),       // Key for storing the admin address
    HasMinted(Address),   // Key for storing whether an address has minted a token
}

// Structure to store minting information
#[contracttype]
#[derive(Clone, Debug)]
pub struct MintTo {
    pub address: Address, // The address to which the NFT is minted
    pub token_id: i128,   // The ID of the minted token
    pub metadata: String, // Metadata associated with the token
    pub image: String,    // Image URL associated with the token
}

// Implementation of the StallionNFT contract
#[contractimpl]
impl StallionNFT {
    // Constants for the NFT
    const NAME: &'static str = "StallionNFT";
    const SYMBOL: &'static str = "SNFT";
    const METADATA: &'static str = "https://ipfs.io/ipfs/bafkreibzw25uz3cxnpd4ditc2s7ngyea2hpq45s7psbs27dm3z6r57rzbe";
    const IMAGE: &'static str = "https://ipfs.io/ipfs/bafybeichocyvocmrrixgunzlrcnj4u7sbg3cst54mp3e3begu4qiphe3jq";
    const SUPPLY: i128 = 2000; // Maximum supply of tokens

    // Constructor to initialize the contract with an admin address
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin(admin.clone()), &admin);
    }

    // Function to get the name of the NFT
    pub fn name(env: Env) -> String {
        String::from_str(&env, Self::NAME)
    }

    // Function to get the symbol of the NFT
    pub fn symbol(env: Env) -> String {
        String::from_str(&env, Self::SYMBOL)
    }

    // Function to get the metadata URI of the NFT
    pub fn token_uri(env: Env) -> String {
        String::from_str(&env, Self::METADATA)
    }

    // Function to get the image URI of the NFT
    pub fn token_image(env: Env) -> String {
        String::from_str(&env, Self::IMAGE)
    }

    // Function to get the owner of a specific token
    pub fn owner_of(env: Env, token_id: i128) -> Address {
        env.storage().persistent().get(&DataKey::Owner(token_id)).unwrap_or_else(|| {
            Address::from_string_bytes(&Bytes::from_slice(&env, &[0; 32]))
        })
    }

    // Function to add an address to the whitelist
    pub fn add_to_whitelist(env: Env, address: Address) {
        let mut whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist).unwrap_or_else(|| Vec::new(&env));
        if whitelist.contains(&address) {
            panic!("Address is already whitelisted");
        }
        whitelist.push_back(address.clone());
        env.storage().persistent().set(&DataKey::Whitelist, &whitelist);
    }

    // Function to get the list of whitelisted addresses
    pub fn get_whitelist(env: Env) -> Vec<Address> {
        env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist).unwrap_or_else(|| Vec::new(&env))
    }

    // Function to remove an address from the whitelist
    pub fn remove_from_whitelist(env: Env, admin: Address, address: Address) {
        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin(admin.clone()))
            .expect("Admin address not set");
        assert_eq!(admin, stored_admin, "Caller is not the admin");

        let mut whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist)
            .expect("Whitelist does not exist");
        if let Some(pos) = whitelist.iter().position(|x| x == address) {
            whitelist.remove(pos.try_into().unwrap());
            env.storage().persistent().set(&DataKey::Whitelist, &whitelist);
        } else {
            panic!("Address not whitelisted");
        }
    }

    // Function to check if an operator is approved for a specific token
    pub fn is_approved(env: Env, operator: Address, token_id: i128) -> bool {
        let key = DataKey::Approvals(token_id);
        let approvals = env.storage().persistent().get::<DataKey, Vec<Address>>(&key).unwrap_or_else(|| Vec::new(&env));
        approvals.contains(&operator)
    }

    // Function to transfer a token from one address to another
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

    // Function to mint a new token to a whitelisted address
    pub fn mint(env: Env, to: Address) {
        let whitelist = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Whitelist)
            .expect("Whitelist not found");
        assert!(whitelist.contains(&to), "Address not whitelisted");

        // Check if the address has already minted a token
        let has_minted = env.storage().persistent().get::<DataKey, bool>(&DataKey::HasMinted(to.clone()))
            .unwrap_or(false);
        assert!(!has_minted, "Address has already minted a token");

        let mut token_count: i128 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);
        assert!(token_count < Self::SUPPLY, "Maximum token supply reached");
        token_count += 1;

        let mint_to = MintTo {
            address: to.clone(),
            token_id: token_count,
            metadata: Self::token_uri(env.clone()),
            image: Self::token_image(env.clone()),
        };

        env.storage().persistent().set(&DataKey::Approvals(token_count), &mint_to);

        env.storage().persistent().set(&DataKey::TokenCount, &token_count);
        env.storage().persistent().set(&DataKey::Owner(token_count), &to);

        // Mark the address as having minted a token
        env.storage().persistent().set(&DataKey::HasMinted(to.clone()), &true);

        env.events().publish((symbol_short!("Mint"),), (to, token_count));
    }

    // Function to retrieve the image URL for a given token ID
    pub fn get_token_image(env: Env, token_id: i128) -> String {
        // Retrieve the MintTo struct from storage and return the image URL
        let mint_to: MintTo = env.storage().persistent().get(&DataKey::Approvals(token_id))
            .expect("MintTo struct not found for this token");
        mint_to.image
    }

    // Function to retrieve the metadata URL for a given token ID
    pub fn get_token_metadata(env: Env, token_id: i128) -> String {
        // Retrieve the MintTo struct from storage and return the metadata URL
        let mint_to: MintTo = env.storage().persistent().get(&DataKey::Approvals(token_id))
            .expect("MintTo struct not found for this token");
        mint_to.metadata
    }

    // Function to approve an address to manage a specific token
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

    // Function to transfer a token from one address to another by an approved spender
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

    // Function to retrieve the NFT associated with a specific address
    pub fn get_nft_by_address(env: Env, address: Address) -> Option<MintTo> {
        let token_count: i128 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);

        for token_id in 1..=token_count {
            if let Some(mint_to) = env.storage().persistent().get::<DataKey, MintTo>(&DataKey::Approvals(token_id)) {
                if mint_to.address == address {
                    return Some(mint_to);
                }
            }
        }
        None
    }
}

mod test;
