# ---------------------------------------------------
# 1. Build Stage
# ---------------------------------------------------
FROM rust:latest AS builder

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build and install the application
# This compiles the binary and moves it to /usr/local/cargo/bin/
RUN cargo install --path .

# ---------------------------------------------------
# 2. Runtime Stage
# ---------------------------------------------------
FROM debian:bookworm-slim

# Install OpenSSL and CA certificates (Verified for Serenity/Discord)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
# We rename it to 'discord-bot' explicitly so CMD is easy to write
COPY --from=builder /usr/local/cargo/bin/ /app/

# Handle the BOT_TOKEN.txt file
# OPTION A: If BOT_TOKEN.txt is in your git repo, this copies it.
# OPTION B: If it is ignored (good practice), you must ensure your deployment platform
#           mounts this file or you create it using a script.
COPY --from=builder /usr/src/app/BOT_TOKEN.txt* /app/

# Set the startup command
CMD ["sh", "-c", "env -u RUSTUP_TOOLCHAIN ./*"]