#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Warranty token structure to track product warranties
#[contracttype]
#[derive(Clone)]
pub struct WarrantyToken {
    pub token_id: u64,
    pub product_name: String,
    pub owner: Address,
    pub issue_date: u64,
    pub expiry_date: u64,
    pub is_active: bool,
}

// Warranty statistics for tracking overall system status
#[contracttype]
#[derive(Clone)]
pub struct WarrantyStats {
    pub total_issued: u64,
    pub active_warranties: u64,
    pub expired_warranties: u64,
    pub claimed_warranties: u64,
}

// Storage keys
const WARRANTY_COUNT: Symbol = symbol_short!("W_COUNT");
const WARRANTY_STATS: Symbol = symbol_short!("W_STATS");

// Mapping token_id to WarrantyToken
#[contracttype]
pub enum WarrantyBook {
    Warranty(u64)
}

#[contract]
pub struct WarrantyContract;

#[contractimpl]
impl WarrantyContract {

    /// Issue a new warranty token for a product
    /// Returns the unique warranty token ID
    pub fn issue_warranty(
        env: Env, 
        owner: Address, 
        product_name: String, 
        warranty_period_days: u64
    ) -> u64 {
        // Require authentication
        owner.require_auth();

        // Get and increment warranty counter
        let mut warranty_count: u64 = env.storage().instance().get(&WARRANTY_COUNT).unwrap_or(0);
        warranty_count += 1;

        // Get current timestamp
        let current_time = env.ledger().timestamp();
        let expiry_date = current_time + (warranty_period_days * 86400); // Convert days to seconds

        // Create warranty token
        let warranty = WarrantyToken {
            token_id: warranty_count,
            product_name,
            owner: owner.clone(),
            issue_date: current_time,
            expiry_date,
            is_active: true,
        };

        // Update statistics
        let mut stats = Self::get_warranty_stats(env.clone());
        stats.total_issued += 1;
        stats.active_warranties += 1;

        // Store warranty token
        env.storage().instance().set(&WarrantyBook::Warranty(warranty_count), &warranty);
        env.storage().instance().set(&WARRANTY_COUNT, &warranty_count);
        env.storage().instance().set(&WARRANTY_STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Warranty Token Issued - ID: {}", warranty_count);
        warranty_count
    }

    /// Check warranty status and validity
    /// Returns the warranty token details
    pub fn check_warranty(env: Env, token_id: u64) -> WarrantyToken {
        let key = WarrantyBook::Warranty(token_id);
        
        env.storage().instance().get(&key).unwrap_or(WarrantyToken {
            token_id: 0,
            product_name: String::from_str(&env, "Not_Found"),
            owner: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            issue_date: 0,
            expiry_date: 0,
            is_active: false,
        })
    }

    /// Claim warranty (mark as used)
    pub fn claim_warranty(env: Env, token_id: u64, claimer: Address) {
        claimer.require_auth();

        let mut warranty = Self::check_warranty(env.clone(), token_id);
        
        // Verify warranty exists and is active
        if warranty.token_id == 0 {
            log!(&env, "Warranty not found!");
            panic!("Warranty not found!");
        }

        if !warranty.is_active {
            log!(&env, "Warranty already claimed or inactive!");
            panic!("Warranty already claimed!");
        }

        // Check if warranty has expired
        let current_time = env.ledger().timestamp();
        if current_time > warranty.expiry_date {
            log!(&env, "Warranty has expired!");
            panic!("Warranty expired!");
        }

        // Verify owner
        if warranty.owner != claimer {
            log!(&env, "Unauthorized: You are not the warranty owner!");
            panic!("Unauthorized!");
        }

        // Mark warranty as claimed
        warranty.is_active = false;

        // Update statistics
        let mut stats = Self::get_warranty_stats(env.clone());
        stats.active_warranties -= 1;
        stats.claimed_warranties += 1;

        // Store updated warranty
        env.storage().instance().set(&WarrantyBook::Warranty(token_id), &warranty);
        env.storage().instance().set(&WARRANTY_STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Warranty claimed - Token ID: {}", token_id);
    }

    /// Get overall warranty statistics
    pub fn get_warranty_stats(env: Env) -> WarrantyStats {
        env.storage().instance().get(&WARRANTY_STATS).unwrap_or(WarrantyStats {
            total_issued: 0,
            active_warranties: 0,
            expired_warranties: 0,
            claimed_warranties: 0,
        })
    }
}