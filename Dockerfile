# Clean-environment non-trace verification image for ecc-audit-engine.
# Trace campaigns requiring Valgrind may use Dockerfile.trace or native Linux.
FROM rust:1.97-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    cmake \
    autoconf \
    automake \
    libtool \
    pkg-config \
    git \
    python3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

# Ensure scripts are executable
RUN chmod +x scripts/*.sh scripts/*.py || true

# Default: non-extended verification (Valgrind optional; full verify needs Dockerfile.trace or host Valgrind)
CMD ["./scripts/verify.sh"]
