// This binary will act as a client to a live Fortuna server instance.
// It will make HTTP requests to request and retrieve random numbers.

use anyhow::{anyhow, Result};
use clap::Parser;
use fortuna::mock_entropy_contract_impl::MockEntropyContract;
use serde::Deserialize;
use sha3::{Digest, Keccak256};
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of random samples to fetch.
    #[arg(long, default_value_t = 100)]
    num_samples: u64,

    /// The chain ID to query (e.g., "ethereum").
    #[arg(long)]
    chain_id: String,

    /// The base URL of the Fortuna server.
    #[arg(long, default_value = "http://localhost:8080")]
    server_url: String,

    /// Path to the fortuna config file.
    #[arg(long, default_value = "config.yaml")]
    config: String,
}

// Minimal structs for deserializing the chain_length from config.yaml
#[derive(Debug, Deserialize)]
struct Config {
    provider: ProviderConfig,
}

#[derive(Debug, Deserialize)]
struct ProviderConfig {
    chain_length: u64,
    chain_sample_interval: u64,
}

// Structs to deserialize the server's JSON response
#[derive(Deserialize, Debug)]
struct GetRandomValueResponse {
    value: Blob,
}

#[derive(Deserialize, Debug)]
struct GetCommitmentResponse {
    commitment: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "encoding", rename_all = "kebab-case")]
enum Blob {
    Hex { data: String },
}

impl Blob {
    fn into_bytes32(self) -> Result<[u8; 32]> {
        match self {
            Blob::Hex { data } => {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(data.trim_start_matches("0x"), &mut bytes)?;
                Ok(bytes)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // --- PRE-FLIGHT CHECK ---
    let config_file = File::open(&args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;
    let max_samples = config.provider.chain_length;

    if args.num_samples > max_samples {
        return Err(anyhow!(
            "Number of samples ({}) cannot exceed the maximum possible samples ({}). To get more samples, increase the chain_length in the config.yaml.",
            args.num_samples,
            max_samples,
        ));
    }

    println!(
        "Performing live black-box test on Fortuna server at {}...",
        args.server_url
    );

    let client = reqwest::Client::new();

    // --- MOCK CONTRACT SETUP ---
    // 1. Fetch the server's actual commitment for the chain.
    let commitment_url = format!("{}/v1/commitment/{}", args.server_url, args.chain_id);
    println!("[CLIENT] Fetching initial commitment from server: {}", commitment_url);
    let commitment_response: GetCommitmentResponse = client.get(&commitment_url).send().await?.json().await?;
    let mut commitment_bytes = [0u8; 32];
    hex::decode_to_slice(&commitment_response.commitment, &mut commitment_bytes)?;
    println!("[SERVER] Reported commitment: {}", commitment_response.commitment);

    // 2. Register the provider on our local mock contract with the real commitment.
    let provider_address = "0xprovider";
    let mut contract = MockEntropyContract::new();
    contract.register(provider_address.to_string(), commitment_bytes);

    // --- In-memory storage for results ---
    let mut random_numbers: Vec<String> = Vec::with_capacity(args.num_samples as usize);
    let mut audit_trail: Vec<String> = Vec::with_capacity(args.num_samples as usize);

    // --- SIMULATION LOOP ---
    for i in 1..=args.num_samples {
        println!("\n--- Sample {} ---", i);

        // 1. Simulate the user generating a secret and committing to it.
        let user_secret: [u8; 32] = Keccak256::digest(format!("user_{}{}", args.chain_id, i)).into();
        let user_commitment = Keccak256::digest(&user_secret).into();

        // 2. Simulate the user submitting the commitment to the contract.
        let sequence_number = contract.request(user_commitment);

        // 3. Call the modified Fortuna server to get its revelation for the sequence number.
        let url = format!(
            "{}/v1/chains/{}/mock_revelation/{}",
            args.server_url, args.chain_id, sequence_number
        );

        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Server returned an error: {} - {}",
                response.status(),
                response.text().await?
            ));
        }
        let revelation_response: GetRandomValueResponse = response.json().await?;
        let provider_revelation = revelation_response.value.into_bytes32()?;

        // 4. Use our local mock contract to fulfill the request using the server's revelation.
        // This verifies the entire cryptographic process.
        let final_random_number = contract.fulfill_request(
            provider_address,
            sequence_number,
            user_secret,
            provider_revelation,
        )?;

        // 5. Store the verified results in memory.
        random_numbers.push(hex::encode(final_random_number));
        audit_trail.push(format!(
            "sample_number={}, sequence_number={}, user_secret={}, provider_revelation={}, final_random_number={}",
            i,
            sequence_number,
            hex::encode(user_secret),
            hex::encode(provider_revelation),
            hex::encode(final_random_number)
        ));
    }

    // --- FILE WRITING ---
    // Write all results to the files at once.
    std::fs::write("../random_numbers.txt", random_numbers.join("\n"))?;
    std::fs::write("../audit_trail.txt", audit_trail.join("\n"))?;

    println!("\n✅ Success! Live audit complete. Files generated: random_numbers.txt, audit_trail.txt");

    Ok(())
} 