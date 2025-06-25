# Fortuna Audit Package for GLI Certification

This package contains a simulation of the `fortuna` entropy provider protocol, designed for auditing and certification by the Gaming Laboratories International (GLI).

The core of this package is a Rust program that faithfully simulates the `Entropy.sol` smart contract's cryptographic logic. It demonstrates how a verifiably random number is generated from a combination of a User's secret and the Provider's secret.

## Package Contents

-   `fortuna/`: Contains the Rust source code for the simulation.
    -   `src/bin/mock_contract.rs`: The main script for generating random numbers.
    -   `smart-contract-explainer.md`: A detailed technical explanation of how the simulation maps to the real smart contract.
-   `target_chains/`: Contains the original `Entropy.sol` smart contract and its dependencies.

## Prerequisites

To run the simulation, you will need the Rust compiler and package manager, `cargo`. If you do not have it installed, you can get it from [rust-lang.org](https://www.rust-lang.org/tools/install).

This project uses a specific Rust version, which is defined in the `fortuna/rust-toolchain` file. The `rustup` tool (which is installed with `cargo`) will automatically download and use this version when you are inside the `fortuna` directory.

## How to Run the Audit

All commands should be run from the `gli-audit-package` directory. The script will generate two files in this same directory:

-   `random_numbers.csv`: A simple list of the final random numbers.
-   `audit_trail.csv`: A detailed log with all inputs and outputs for each sample.

To generate the audit files, run the following commands:

```bash
cd fortuna
cargo run --release --bin mock_contract -- --num-samples 1000
```

You can change the `--num-samples` argument to generate any number of samples you require. 