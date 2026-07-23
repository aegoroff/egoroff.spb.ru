# egoroff.spb.ru task runner
#
# Backend workspace lives in ./egoroff; recipes use --manifest-path so they
# can be invoked from the repo root. Frontend recipes operate in ./ui.

# Show available recipes
default:
    @just --list

# ===== Backend (Rust) =====

# Build the workspace (debug)
[group('backend')]
build:
    cargo build --manifest-path egoroff/Cargo.toml

# Build the workspace (release)
[group('backend')]
build-release:
    cargo build --manifest-path egoroff/Cargo.toml --release

# Run all tests
[group('backend')]
test:
    cargo test --manifest-path egoroff/Cargo.toml

# Lint the workspace with pedantic clippy
[group('backend')]
clippy:
    cargo clippy --manifest-path egoroff/Cargo.toml --workspace -- -W clippy::pedantic

# Format the Rust sources
[group('backend')]
fmt:
    cargo fmt --manifest-path egoroff/Cargo.toml

# Remove build artifacts
[group('backend')]
clean:
    cargo clean --manifest-path egoroff/Cargo.tomlW

# ===== Frontend (Vue + bun, in ./ui) =====

# Install frontend dependencies
[group('frontend')]
install:
    cd ui && bun install

# Production frontend build -> static/dist/
[group('frontend')]
ui-build:
    cd ui && bun run buildW

# ESLint check
[group('frontend')]
lint:
    cd ui && bun run lint

# ===== Apache documentation (XSLT, optional) =====

# Compile Apache XSLT documentation into templates/apache/
[group('docs')]
apache:
    python3 build.py

# Clean generated Apache docs, then rebuild
[group('docs')]
apache-clean:
    python3 build.py --clean-all

# ===== Local server run (make_local.sh, as-is) =====

# Build UI + backend and run the local server.
# Usage: just local [debug|release]
[group('local')]
local profile="debug":
    #!/usr/bin/env bash
    if [[ "{{profile}}" = "release" ]]; then
      additional="--release"
    else
      additional=""
    fi
    base_path=./home
    [[ -d "$base_path" ]] && rm -r "$base_path"
    (
      cd ./ui/ || exit
      bun run build || exit
    )
    mkdir "$base_path"
    locals=(static apache)
    for local in "${locals[@]}"; do
      cp -v -R "./$local/" "$base_path/$local/"
    done
    (
      cd ./egoroff/ || exit
      cargo clean
      cargo b --workspace $additional
      RUST_LOG="server=debug,axum=info,hyper=info,tower=info,axum_login=info" ./target/"{{profile}}"/egoroff server
    )

# ===== Docker (docker.sh, as-is: buildx multi-arch publish) =====

# Build + publish amd64/arm64 images and a multi-arch manifest via buildx.
# Usage: just docker [tag]   (or set TAG env var)
[group('docker')]
docker tag=env("TAG", "master"):
    #!/usr/bin/env bash
    set -euo pipefail
    full_tag="registry.egoroff.spb.ru/egoroff/egoroff.spb.ru:{{tag}}"
    builder="egoroff-multiarch"

    if ! docker buildx inspect "${builder}" >/dev/null 2>&1; then
      docker buildx create --name "${builder}" --driver docker-container --use
    else
      docker buildx use "${builder}"
    fi

    # Per-arch images. --provenance=false keeps a plain image manifest (no attestation index).
    docker buildx build \
      --platform linux/amd64 \
      -t "${full_tag}-x64" \
      --provenance=false \
      --push \
      .

    docker buildx build \
      --platform linux/arm64 \
      -t "${full_tag}-arm64" \
      --provenance=false \
      --push \
      .

    # registry:3 + buildx imagetools often fails with HEAD 400 on an existing tag.
    # docker manifest list (Schema 2) is what this registry already accepts.
    docker manifest rm "${full_tag}" 2>/dev/null || true
    docker manifest create "${full_tag}" "${full_tag}-x64" "${full_tag}-arm64"
    docker manifest push --purge "${full_tag}"

    echo "Pushed multi-arch manifest: ${full_tag}"
    docker buildx imagetools inspect "${full_tag}"

# ===== Umbrella checks =====

# Full pre-commit check: clippy + tests + frontend lint
check: clippy test lint
