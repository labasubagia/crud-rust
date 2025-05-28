# Run as a non-privileged user
FROM rust:1.87
RUN useradd -ms /bin/sh -u 1001 app
USER app

# Install dependencies
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Add dummy source file to make manifest valid
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Pre-fetch dependencies
RUN cargo fetch
# Clean up dummy src to avoid conflict with real code
RUN rm -rf src

# Copy source files into application directory
COPY --chown=app:app . /app