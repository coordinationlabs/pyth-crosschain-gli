// This module contains the logic to simulate the Entropy.sol smart contract in memory.
// It is used by both the offline `mock_contract` simulation and the `live_client` integration test.

use anyhow::{ensure, Result};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

// --- Mock Contract State ---
// These structs mimic the state stored in the real Entropy.sol contract.

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// The provider's current request counter.
    pub sequence_number: u64,
    /// The hash the provider initially committed to.
    pub commitment: [u8; 32],
    /// The last revealed hash from the provider. Used for verification.
    pub last_revealed_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct Request {
    pub user_commitment: [u8; 32],
}

/// A mock of the Entropy.sol smart contract that manages provider state and fulfills requests.
pub struct MockEntropyContract {
    /// Mapping of provider address to their info
    providers: HashMap<String, ProviderInfo>,
    /// Mapping of a global sequence number to a user's request
    requests: HashMap<u64, Request>,
}

impl MockEntropyContract {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            requests: HashMap::new(),
        }
    }

    /// Mimics the `register` function in the smart contract.
    /// Stores the provider's initial commitment (the last hash in their chain).
    pub fn register(&mut self, provider_address: String, commitment: [u8; 32]) {
        self.providers.insert(
            provider_address,
            ProviderInfo {
                sequence_number: 0,
                commitment,
                last_revealed_hash: commitment,
            },
        );
    }

    /// Mimics the `request` function. Stores the user's commitment.
    pub fn request(&mut self, user_commitment: [u8; 32]) -> u64 {
        let provider_info = self.providers.values_mut().next().unwrap();
        provider_info.sequence_number += 1;
        let sequence_number = provider_info.sequence_number;

        self.requests.insert(sequence_number, Request { user_commitment });
        sequence_number
    }

    /// Mimics the `fulfill` function. Verifies the provider's revelation and generates the final number.
    pub fn fulfill_request(
        &mut self,
        provider_address: &str,
        sequence_number: u64,
        user_secret: [u8; 32],
        provider_revelation: [u8; 32],
    ) -> Result<[u8; 32]> {
        let provider_info = self.providers.get_mut(provider_address).unwrap();
        let request = self.requests.get_mut(&sequence_number).unwrap();

        // --- VERIFICATION LOGIC (mirrors Entropy.sol) ---

        // 1. Verify the user's secret matches their commitment.
        let user_commitment_check: [u8; 32] = Keccak256::digest(&user_secret).into();
        ensure!(
            request.user_commitment == user_commitment_check,
            "User commitment verification failed"
        );

        // 2. Verify the provider's revelation hashes to their last revealed hash.
        let provider_commitment_check: [u8; 32] =
            Keccak256::digest(&provider_revelation).into();
        ensure!(
            provider_info.last_revealed_hash == provider_commitment_check,
            "Provider commitment verification failed"
        );

        // Update the provider's last revealed hash for the next fulfillment.
        provider_info.last_revealed_hash = provider_revelation;

        // 3. Combine secrets to generate the final random number.
        let final_random_number = Keccak256::digest([user_secret, provider_revelation].concat()).into();
        Ok(final_random_number)
    }
} 