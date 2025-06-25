# Fortuna (pyth-crosschain-gli) Setup Instructions

This guide provides a simplified, step-by-step process to run the Fortuna server using Docker. It assumes you are starting with a version of the repository that includes the necessary fixes to the `Dockerfile` and `config.yaml`.

## Prerequisites

- You must have **Docker** installed and running.
  - [Download Docker Desktop](https://www.docker.com/products/docker-desktop/)
- You must have **Foundry** installed to generate wallets.
  - If you don't have it, run these two commands:
    ```bash
    curl -L https://foundry.paradigm.xyz | bash
    foundryup
    ```
    *(You may need to restart your terminal after installation for the commands to be available.)*

## Step 1: Generate Wallets and Secrets

The server needs unique cryptographic keys to run.

1.  **Generate a Provider Wallet, a Keeper Wallet, and a Secret:**
    Run the following command in your terminal and copy the output:

    ```bash
    echo "Provider Wallet:" && cast wallet new && echo "\nKeeper Wallet:" && cast wallet new && echo "\nSecret:" && openssl rand -hex 32
    ```
    Your output will look similar to this, providing the addresses and keys you'll need for the next step:
    ```
    Provider Wallet:
    Successfully created new keypair.
    Address:     0xAbC123...
    Private key: 0xdeadbeef...

    Keeper Wallet:
    Successfully created new keypair.
    Address:     0xDeF456...
    Private key: 0xfeedface...

    Secret:
    a1b2c3d4e5f6...
    ```

## Step 2: Update the Configuration File

1.  **Open `apps/fortuna/config.yaml`**.
2.  **Replace the placeholder values** with the addresses, private keys, and secret you just generated.
    - Replace `0xADDRESS_HERE` with your **Provider Address**.
    - Replace `0xPRIVATE_KEY_HERE` with the corresponding **Private Key**.
    - Do this for both the `provider` and `keeper` sections.
    - Replace `0xSECRET_HERE` with the **Secret** you generated.

## Step 3: Build and Run the Server

1.  **Build the Docker image:**
    From the root of the project directory, run:
    ```bash
    docker build -t fortuna-server -f apps/fortuna/Dockerfile .
    ```

2.  **Run the server:**
    Once the build is complete, run the following command:
    ```bash
    docker run --rm -it -p 34000:34000 -v $(pwd)/apps/fortuna/config.yaml:/config.yaml fortuna-server /usr/local/bin/fortuna run --config /config.yaml
    ```

The server should now be running. You can access it on `localhost:34000`. 