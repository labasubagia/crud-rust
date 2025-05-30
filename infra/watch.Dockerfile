FROM rust:1.87

# Add mold for faster builds
RUN apt-get update && apt-get install -y mold

# Run as a non-privileged user
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

# Create target with correct permissions
RUN mkdir -p target && chown -R app:app target

# Copy source files into application directory
COPY --chown=app:app . /app