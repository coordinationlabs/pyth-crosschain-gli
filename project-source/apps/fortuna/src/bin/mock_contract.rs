use anyhow::{ensure, Result};
use fortuna::state::PebbleHashChain;
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

// --- Mock Contract State ---
// These structs mimic the state stored in the real Entropy.sol contract.

#[derive(Debug, Clone)]
struct ProviderInfo {
    /// The provider's current request counter.
    sequence_number: u64,
    /// The hash the provider initially committed to.
    commitment: [u8; 32],
    /// The last revealed hash from the provider. Used for verification.
    last_revealed_hash: [u8; 32],
}

#[derive(Debug, Clone)]
struct Request {
    requester: String,
    user_commitment: [u8; 32],
}

/// A mock of the Entropy.sol smart contract that manages state in memory.
struct MockEntropyContract {
    /// Mapping of provider address to their info
    providers: HashMap<String, ProviderInfo>,
    /// Mapping of a global sequence number to a user's request
    requests: HashMap<u64, Request>,
}

impl MockEntropyContract {
    fn new() -> Self {
        Self {
            providers: HashMap::new(),
            requests: HashMap::new(),
        }
    }

    /// Mimics the `register` function in the smart contract.
    /// Stores the provider's initial commitment (the last hash in their chain).
    fn register(&mut self, provider_address: String, commitment: [u8; 32]) {
        println!("[CONTRACT] Provider '{}' registered with commitment: {}", provider_address, hex::encode(commitment));
        self.providers.insert(
            provider_address,
            ProviderInfo {
                sequence_number: 0,
                commitment,
                last_revealed_hash: commitment,
            },
        );
    }

    /// Mimics the `requestRandomness` function.
    /// Takes a user's commitment, assigns a sequence number, and stores the request.
    fn request_randomness(&mut self, provider_address: &str, user_address: String, user_commitment: [u8; 32]) -> Result<u64> {
        let provider_info = self.providers.get_mut(provider_address).ok_or_else(|| anyhow::anyhow!("Provider not found"))?;

        let assigned_sequence_number = provider_info.sequence_number;
        println!("[CONTRACT] Received request from '{}' for provider '{}'. Assigning sequence number: {}", user_address, provider_address, assigned_sequence_number);

        self.requests.insert(assigned_sequence_number, Request {
            requester: user_address,
            user_commitment,
        });

        // Increment the provider's sequence number for the next request
        provider_info.sequence_number += 1;

        Ok(assigned_sequence_number)
    }

    /// Mimics the `reveal` function.
    /// This is the core verification logic.
    fn reveal(&mut self, provider_address: &str, sequence_number: u64, provider_revelation: [u8; 32], user_secret: [u8; 32]) -> Result<[u8; 32]> {
        println!("[CONTRACT] Received reveal for sequence number: {}", sequence_number);

        let provider_info = self.providers.get_mut(provider_address).ok_or_else(|| anyhow::anyhow!("Provider not found"))?;
        let request = self.requests.get(&sequence_number).ok_or_else(|| anyhow::anyhow!("Request not found"))?;

        // 1. Verify the provider's revelation.
        //    Hash the number they just revealed (`provider_revelation`).
        //    The result must equal the *last* revealed hash they submitted.
        let a_hash_of_the_revelation: [u8; 32] = Keccak256::digest(provider_revelation).into();
        ensure!(
            a_hash_of_the_revelation == provider_info.last_revealed_hash,
            "Provider revelation is invalid! Chain is broken."
        );
        println!("[CONTRACT] Provider revelation is VALID.");

        // 2. Verify the user's secret.
        //    Hash the secret the user provided in this call.
        //    The result must equal the commitment the user made in their initial request.
        let a_hash_of_the_user_secret: [u8; 32] = Keccak256::digest(user_secret).into();
        ensure!(
            a_hash_of_the_user_secret == request.user_commitment,
            "User secret is invalid!"
        );
        println!("[CONTRACT] User secret is VALID.");

        // 3. Both parties are honest. Update the provider's state for the next reveal.
        provider_info.last_revealed_hash = provider_revelation;

        // 4. Combine the secrets to produce the final random number.
        let final_random_number = Keccak256::digest([user_secret, provider_revelation].concat()).into();
        println!("[CONTRACT] Successfully generated final random number.");
        
        Ok(final_random_number)
    }
}


fn main() -> Result<()> {
    let num_samples = 1000;
    println!("Generating {} random samples for audit...", num_samples);
    println!("-------------------------------------------------");

    // --- 1. SETUP ---
    // The provider generates their secret and the full hash chain that will be used for all samples.
    let provider_address = "auditable_provider_1".to_string();
    let provider_secret = [1u8; 32]; // A fixed, known secret for reproducibility.
    let chain_length = num_samples + 1; // Length must be enough for all samples + initial commitment.
    let provider_hash_chain = PebbleHashChain::new(provider_secret, chain_length, 10);
    println!("[PROVIDER] Generated a hash chain of length {}.", chain_length);

    // The mock contract is "deployed".
    let mut contract = MockEntropyContract::new();
    println!("[CONTRACT] MockEntropyContract has been deployed.");

    // --- 2. REGISTRATION ---
    // The provider registers their commitment with the contract. This is the "top" of the hash chain.
    let provider_commitment = provider_hash_chain.reveal_ith(0)?;
    contract.register(provider_address.clone(), provider_commitment);
    println!("-------------------------------------------------");


    // --- 3. SIMULATION LOOP ---
    for i in 0..num_samples {
        println!("\n--- Sample #{} ---", i + 1);

        // A. USER REQUESTS RANDOMNESS
        // In a real scenario, each request would have a new secret. We'll simulate this
        // by creating a unique secret for each iteration.
        let user_secret: [u8; 32] = Keccak256::digest(i.to_be_bytes()).into();
        let user_commitment: [u8; 32] = Keccak256::digest(user_secret).into();
        
        // The user submits their commitment to the contract.
        let sequence_number = contract.request_randomness(
            &provider_address,
            format!("auditable_user_{}", i), // A unique user for each request
            user_commitment
        )?;

        // B. PROVIDER REVEALS
        // The provider sees the request for `sequence_number` and gets the corresponding hash from their chain.
        // The sequence number from the contract corresponds to `sequence_number + 1` in the hash chain's indices,
        // because index 0 was the initial commitment.
        let provider_revelation = provider_hash_chain.reveal_ith((sequence_number + 1) as usize)?;

        // C. CONTRACT VERIFIES AND GENERATES FINAL NUMBER
        // The provider submits their revelation and the user's original secret to the contract.
        // The contract verifies both and combines them.
        let final_random_number = contract.reveal(
            &provider_address,
            sequence_number,
            provider_revelation,
            user_secret
        )?;

        // D. AUDITABLE OUTPUT
        println!("\n--- Audit Trail for Sample #{} ---", i + 1);
        println!("User Secret Input:      {}", hex::encode(user_secret));
        println!("Provider Reveal Input:  {}", hex::encode(provider_revelation));
        println!("Final Random Output:    {}", hex::encode(final_random_number));
    }

    println!("\n-------------------------------------------------");
    println!("Successfully generated and verified {} samples.", num_samples);

    Ok(())
} 