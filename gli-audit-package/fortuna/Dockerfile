# Stage 1: Build the application
# Use the specific Rust version from the rust-toolchain file for consistency.
FROM rust:1.82.0 as builder

# Set the base working directory.
WORKDIR /src

# Copy the required source directories into the container,
# preserving the project's directory structure which is essential
# for the relative path dependencies in Cargo.toml to resolve correctly.
COPY ./gli-audit-package /src/gli-audit-package
COPY ./project-source /src/project-source

# Set the working directory to our specific application.
WORKDIR /src/gli-audit-package/fortuna

# Build the Fortuna binary.
# The --mount flag caches dependencies to speed up subsequent builds.
# The SQLX_OFFLINE=true flag is crucial for building without a database connection.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    SQLX_OFFLINE=true cargo build --release --bin fortuna

# Stage 2: Create the final runtime image
# We use the same Rust image as the builder to ensure that all necessary
# system-level shared libraries (like libssl) are present.
FROM rust:1.82.0

# Copy the compiled Fortuna binary from the builder stage.
COPY --from=builder /src/gli-audit-package/fortuna/target/release/fortuna /usr/local/bin/

# Expose the port the server listens on.
EXPOSE 8080

# Set the final working directory and the command to run the application.
# The config file is expected to be mounted at /app/config.yaml at runtime.
WORKDIR /app
CMD ["fortuna", "run", "--config", "config.yaml", "--rpc-listen-addr", "0.0.0.0:8080"]
