# Build:
# docker build -t simplicity-wasm .
#
# Run (Development):
# docker run -d -p 8080:80 --name simplicity-wasm simplicity-wasm
#
# Run (Production with custom domain):
# docker run -d -p 8080:80 -e VIRTUAL_HOST=your-domain.com --name simplicity-wasm simplicity-wasm
#
# View logs:
# docker logs -f simplicity-wasm
#
# Stop and remove:
# docker stop simplicity-wasm && docker rm simplicity-wasm

# Stage 1: Builder
# This stage installs all dependencies and builds the WASM application.
FROM debian:bookworm-slim AS builder

# Install essential build tools, clang, and nodejs for Trunk
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    clang-16 \
    llvm-16 \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    nodejs \
    npm && \
    rm -rf /var/lib/apt/lists/*

# Install Rust and the wasm32 target
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    rustup target add wasm32-unknown-unknown

# Install Trunk (WASM bundler)
RUN cargo install trunk

# Set environment variables for WASM C compiler
# This tells Rust's build scripts to use clang-16 when compiling C code for Wasm
ENV CC_wasm32_unknown_unknown=clang-16
ENV AR_wasm32_unknown_unknown=llvm-ar-16
ENV CFLAGS_wasm32_unknown_unknown="-I/usr/lib/clang/16/include"

# Copy the application source code
WORKDIR /app
COPY . .

# Build the application in release mode (optimized)
# Output: dist/ directory with optimized WASM bundle (~700KB gzipped)
RUN trunk build --release

# Stage 2: Final Image
# This stage creates a minimal image to serve the built static files via Nginx
FROM nginx:1.27-alpine-slim

# Install wget for healthcheck
RUN apk add --no-cache wget

# Update nginx configuration for SPA routing with caching strategy
# Ensures all routes fall back to index.html for client-side routing
RUN echo 'server { \
    listen 80; \
    server_name _; \
    root /usr/share/nginx/html; \
    index index.html index.htm; \
    client_max_body_size 10M; \
    \
    # Cache busting for versioned assets (1 year) \
    location ~* \.(js|css|wasm)$ { \
        expires 1y; \
        add_header Cache-Control "public, immutable"; \
    } \
    \
    # SPA routing: serve index.html for all non-static routes \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    \
    # Disable caching for HTML files \
    location ~* \.html?$ { \
        expires -1; \
        add_header Cache-Control "no-cache, no-store, must-revalidate"; \
    } \
    \
    # Security headers \
    add_header X-Frame-Options "SAMEORIGIN" always; \
    add_header X-Content-Type-Options "nosniff" always; \
    add_header X-XSS-Protection "1; mode=block" always; \
}' > /etc/nginx/conf.d/default.conf

# Copy the built assets from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Create a healthcheck endpoint
# Checks if the application is responding at http://localhost/index.html
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/index.html || exit 1

# Expose port 80 (HTTP)
EXPOSE 80

# Start Nginx in foreground mode (required for Docker)
CMD ["nginx", "-g", "daemon off;"]
