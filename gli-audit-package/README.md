# Fortuna Audit Package for GLI Certification

This package contains tools to audit the `fortuna` entropy provider protocol, designed for certification by the Gaming Laboratories International (GLI).

The audit is performed via a **Live Black-Box Test**. This involves running a test-enabled instance of the `fortuna` server (included in this package) and using the provided `live_client` to verify its responses. This provides a complete, end-to-end integration test of the server's core cryptographic logic over a network interface, without requiring live blockchain transactions.

## Test-Enabled Server

The `fortuna` server within this audit package has been modified for testing purposes:

-   A new endpoint `GET /v1/commitment/:chain_id` was added to expose the server's initial hash chain commitment. This is used by the client to initialize its verification logic.
-   A new endpoint `GET /v1/chains/:chain_id/mock_revelation/:sequence` was added to reveal numbers from the hash chain without requiring an on-chain transaction.
-   These modifications can be reviewed in `fortuna/src/api/`. The original, unmodified server code is available for reference in the `project-source` directory.

See the Server Modifications for Testing section for more detailed discussion of modifications.

## How to Run the Audit

The audit process uses Docker to ensure a clean, consistent, and easy-to-run test environment. It involves running the test-enabled server in a Docker container and then running a `live_client` against it from your local machine.

### Step 1: Build the Docker Image

The `Dockerfile` included in this package (`fortuna/Dockerfile`) is designed to build the Fortuna server and all its dependencies.

From the **root of the `pyth-crosschain-gli` repository**, run the following command:

```bash
docker build -t fortuna-audit -f gli-audit-package/fortuna/Dockerfile .
```
*   **`-f`**: Specifies the path to the correct `Dockerfile`.
*   **`.`**: Sets the build context to the repository root, which is required for the `Dockerfile` to copy all necessary source code.

### Step 2: Run the Fortuna Server Container

Once the image is built, run it in a container. This command will start the server in the background and mount the local `config.yaml` file into the container.

```bash
# Make sure to run this from the repository root
docker run -d -p 8080:8080 --name fortuna-server \
  -v "$(pwd)/gli-audit-package/fortuna/config.yaml:/app/config.yaml" \
  fortuna-audit
```
*   **`-d`**: Runs the container in detached mode.
*   **`-p 8080:8080`**: Maps port 8080 on your local machine to port 8080 inside the container.
*   **`-v`**: Mounts the local `config.yaml` into the container, allowing the server to use it for configuration.

You can check that the server is running with `docker ps`.

### Step 3: Run the Live Audit Client

With the server running in Docker, you can now run the `live_client`. This client will connect to the server, request random numbers, and perform a full cryptographic verification of the server's responses.

Navigate to the `fortuna` directory and run the client:
```bash
# Change to the application directory
cd gli-audit-package/fortuna

# Run the client
SQLX_OFFLINE=true cargo run --release --bin live_client -- --num-samples 100 --chain-id ethereum
```
*   **`SQLX_OFFLINE=true`**: Is required to build the client without needing a live database connection.
*   **`--server-url`**: If your server is not on `localhost:8080`, you can add this flag.

### Step 4: Verify Output and Cleanup

The client will print its progress for each of the 100 samples. Upon successful completion, you will see the message:

`✅ Success! Live audit complete. Files generated: random_numbers.txt, audit_trail.txt`

The two text files will be generated in the `gli-audit-package` directory.
*   **`random_numbers.txt`**: File containing sampled random numbers, one on each row.
*   **`audit_trail.txt`**: File with associated metadata used to generate the above random numbers such as user and provider commitments and chain sequence numbers.

To stop and remove the Docker container after the audit, run:
```bash
docker stop fortuna-server
docker rm fortuna-server
```

## Server Modifications for Testing

To enable the live black-box test, the `fortuna` server within this audit package was modified with two test-only API endpoints. These endpoints expose the core cryptographic functions of the server over an HTTP interface, allowing a client to verify the server's behavior without needing a live blockchain.

The original, unmodified server code is available for reference in the `project-source` directory. The modifications can be reviewed in `fortuna/src/api/`.

### `GET /v1/commitment/:chain_id`

-   **Purpose**: Exposes the server's initial hash chain commitment for a given chain.
-   **Why it's needed**: The test client must know the server's commitment to correctly initialize its local simulation of the `Entropy.sol` contract. This ensures the client is validating against the same cryptographic state that the server is using.
-   **Response**: A JSON object containing the hex-encoded 32-byte commitment.
    ```json
    {
      "commitment": "0x..."
    }
    ```

### `GET /v1/chains/:chain_id/mock_revelation/:sequence`

-   **Purpose**: Reveals a number from the server's hash chain for a specific sequence number.
-   **Why it's needed**: This endpoint simulates the "reveal" step of the protocol in a controlled manner. It allows the test client to request a value for any sequence number and receive the server's corresponding revelation immediately. This bypasses the need for an actual on-chain request, RPC nodes, and transaction fees, isolating the test to the server's cryptographic logic.
-   **Response**: A JSON object containing the hex-encoded 32-byte revealed value.
    ```json
    {
      "value": {
        "encoding": "hex",
        "data": "0x..."
      }
    }
    ```

### The `local_audit` Flag

To facilitate a true black-box audit without external dependencies, a `local_audit: true` flag can be set in the `provider` section of the `config.yaml`. This is the recommended and required way to run the server for this audit.

When this flag is enabled, the Fortuna server's startup behavior is modified to ensure a completely hermetic testing environment:

-   **On-Chain State is Ignored**: The server **does not** query the blockchain for any existing provider information or commitment state. This removes any dependency on a live network or pre-existing contract state.

-   **Local Commitment Generation**: The server generates a brand new `PebbleHashChain` and its corresponding root commitment **from scratch**, using only the `secret` and `chain_length` values provided in the local `config.yaml` file. This locally generated commitment becomes the root of trust for the audit.

-   **Keeper Service is Disabled**: The Keeper, which is the component responsible for all on-chain transactions (listening for requests and revealing numbers), is **not started**. This is because its functionality is replaced by the `GET /v1/mock_revelation/:sequence` test endpoint for the duration of the audit.

These changes ensure that the `live_client` is auditing a fresh, predictable instance of the server whose cryptographic behavior is based *only* on the provided configuration.

## Technical Explainer

For a detailed technical explanation of how the verification logic in the client maps to the real smart contract, please see `fortuna/smart-contract-explainer.md`. 