# Explaining the `mock_contract.rs` Simulation

This document explains how the Rust script located at `src/bin/mock_contract.rs` serves as a faithful, local simulation of the on-chain interactions with the `Entropy.sol` smart contract. Its purpose is to demonstrate the core cryptographic protocol for gaming certification and auditing, ensuring confidence that the generated randomness is secure and verifiable.

## Core Concepts

The system is based on a **commit-reveal** protocol involving three parties:

1.  **The User:** An application (e.g., a smart contract for a game) that needs a random number.
2.  **The Provider (`fortuna`):** A server that has pre-generated a long, deterministic sequence of secret numbers (the `PebbleHashChain`).
3.  **The `Entropy.sol` Contract:** The on-chain referee that validates the inputs from both the User and the Provider and combines them to produce the final random number.

The key security principle is that the final number is provably random as long as *either* the User or the Provider is honest and keeps their secret value hidden until the final step.

## Mapping the Mock Contract to `Entropy.sol`

Our `mock_contract.rs` script simulates the logic of the `Entropy.sol` contract by managing its state in memory and replicating its core functions.

---

### 1. State Management

The mock contract holds the same essential pieces of information as the real contract.

| `mock_contract.rs` (Rust Simulation)                                                                | `Entropy.sol` (Real Smart Contract)                                                                    | Explanation                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `struct ProviderInfo { sequence_number, commitment, last_revealed_hash }`                           | `struct ProviderInfo { sequenceNumber, originalCommitment, currentCommitment, ... }` in `EntropyState.sol` | This struct holds the state for each registered provider. Our mock focuses on the essential fields for the protocol: the provider's request counter and their cryptographic commitments.                     |
| `struct Request { user_commitment, ... }`                                                           | `struct Request { commitment, ... }` in `EntropyState.sol`                                             | This struct holds the user's part of the commitment for a specific request. In the real contract, this also includes the provider's commitment combined, but the principle is identical.                 |
| `providers: HashMap<String, ProviderInfo>`                                                          | `mapping(address => EntropyStructsV2.ProviderInfo) internal _providers;` in `EntropyState.sol`         | A mapping from a provider's unique identifier (their address in the real contract) to their state. Our simulation uses a `HashMap` for this.                                                            |
| `requests: HashMap<u64, Request>`                                                                   | `mapping(bytes32 => EntropyStructsV2.Request) internal requestsOverflow;` in `EntropyState.sol`         | A mapping to store the details of an active user request. The real contract uses a clever array/mapping hybrid for gas efficiency, but the logic is the same: store the request details for later verification. |

---

### 2. Core Functions and Logic

Each major step in the simulation maps directly to a function in `Entropy.sol`.

| `mock_contract.rs` Function                                | `Entropy.sol` Function                                                                                                                              | Explanation                                                                                                                                                                                                                                                                                                 |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `register(provider_address, commitment)`                   | `register(feeInWei, commitment, ...)`                                                                                                               | **Provider Registration:** In both, a provider registers by submitting their initial commitment. In our simulation, this is the "top" hash (index 0) of their `PebbleHashChain`. The contract stores this value as the anchor for all future verifications from this provider.                                |
| `request_randomness(provider, user, user_commitment)`      | `requestV2(provider, userContribution, ...)` which calls the internal `requestHelper(...)`                                                           | **User Request:** A user submits their commitment (a hash of their secret). The contract assigns a `sequenceNumber` by incrementing the provider's internal counter (`providerInfo.sequenceNumber++`) and stores the user's commitment, linking it to the assigned sequence number. Our mock does the exact same. |
| `reveal(provider, sequence, provider_revelation, user_secret)` | `revealWithCallback(provider, sequenceNumber, userContribution, providerContribution)` which calls the internal `revealHelper(...)` | **Verification and Finalization:** This is the most critical step. The logic is identical in both the simulation and the real contract:                                                                                                                                                                 |
| **1. Verify Provider**                                     | `constructProviderCommitment(...)` and `if (keccak256(...) != req.commitment)`                                                                       | The mock contract hashes the `provider_revelation`. The result **must** equal the `last_revealed_hash` from the previous interaction. This proves the provider is correctly following their pre-committed hash chain.                                                                                    |
| **2. Verify User**                                         | `constructUserCommitment(...)` and `if (keccak256(...) != req.commitment)`                                                                           | The mock contract hashes the `user_secret`. The result **must** equal the `user_commitment` that the user submitted in their initial request. This proves the user is revealing the secret they originally committed to.                                                                               |
| **3. Generate Final Number**                               | `combineRandomValues(...)`                                                                                                                          | If both checks pass, the mock contract combines the `user_secret` and the `provider_revelation` by hashing them together. This produces the final, verifiably random number. This is identical to how the real contract generates the output.                                                          |

## Conclusion

The `mock_contract.rs` script is a high-fidelity simulation of the `Entropy.sol` smart contract's core cryptographic logic. It correctly implements the state management, verification steps, and cryptographic hashing (using the same `Keccak256` algorithm) as the on-chain version. The audit trail generated by this script provides a transparent and accurate representation of the random number generation process, suitable for verification and certification purposes. 