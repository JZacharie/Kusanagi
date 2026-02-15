# Kusanagi CI/CD Guide

## Quick Build (Local)

```bash
# Full build without cache
./ci-build.sh v0.3.0

# With registry prefix
./ci-build.sh v0.3.0 ghcr.io/username/
```

## GitHub Actions

Automatically builds on push to `main` or `master`.

Required setup:
1. Go to **Settings > Actions > General**
2. Enable "Read and write permissions" for workflows

The workflow:
- Builds Rust binary
- Builds Docker image with `--no-cache`
- Verifies old `k8s.js` is absent
- Verifies new modules are present
- Pushes to GHCR

## GitLab CI

Automatically builds on every commit.

Required variables:
- `CI_REGISTRY_USER` (auto-set)
- `CI_REGISTRY_PASSWORD` (auto-set)

Stages:
1. `build:rust` - Compile binary
2. `docker:build` - Build and push image
3. `verify:image` - Verify structure

## Manual Docker Build

```bash
# 1. Clean
cargo clean
rm -f static/js/k8s.js

# 2. Build Rust
cargo build --release

# 3. Build Docker (NO CACHE)
docker build \
  --no-cache \
  --target release-ci \
  --build-arg PREBUILT_BINARY=target/release/kusanagi \
  -t kusanagi:v0.3.0 .

# 4. Verify
docker run --rm kusanagi:v0.3.0 test ! -f /app/static/js/k8s.js
docker run --rm kusanagi:v0.3.0 ls /app/static/js/k8s/
```

## Common Issues

### "Old k8s.js still present!"

The Docker cache contains old files. Solutions:
1. Use `--no-cache` flag
2. Use `docker system prune -a` (nuclear option)
3. Use `./ci-build.sh` which handles this

### "Binary not found"

The `.dockerignore` excludes `target/`. Fixed by:
```dockerignore
# Allow pre-built binary for CI builds
!target/release/kusanagi
```

### Wrong static files in image

The `runner` stage caches old static. Fixed in Dockerfile:
```dockerfile
FROM runner AS release-ci
# Remove any cached static from base image
RUN rm -rf /app/static/* 2>/dev/null || true
COPY static ./static
```

## Deploy

```bash
# Helm
echo "image:
  repository: ghcr.io/username/kusanagi
  tag: v0.3.0
  pullPolicy: Always" | helm upgrade --install kusanagi ./helmscharts/charts/kusanagi -f -

# Or kubectl
docker push ghcr.io/username/kusanagi:v0.3.0
kubectl set image deployment/kusanagi kusanagi=ghcr.io/username/kusanagi:v0.3.0
kubectl rollout restart deployment/kusanagi
```
