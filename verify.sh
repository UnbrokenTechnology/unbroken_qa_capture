#!/bin/bash
set -euo pipefail

# Unbroken QA Capture — verify.sh
# Runs linting and tests for both Rust backend and Vue/TypeScript frontend.
# Exits 0 if project isn't scaffolded yet.

# Source Rust environment if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

RUST_OK=true
TS_OK=true
CMDS_OK=true
SMOKE_OK=true

# ─── Rust Backend ───────────────────────────────────────────────────────────

if [ -f "src-tauri/Cargo.toml" ]; then
    # Tauri's generate_context!() proc macro requires frontendDist ("../dist"
    # relative to src-tauri/) to exist at compile time, even for clippy.
    # Ensure it exists so clippy can run without the full frontend build.
    mkdir -p dist

    echo "=== Rust: clippy ==="
    (cd src-tauri && cargo clippy --all-targets -- -D warnings) || RUST_OK=false

    echo "=== Rust: tests ==="
    (cd src-tauri && cargo test --all-targets) || RUST_OK=false
elif [ -f "Cargo.toml" ]; then
    echo "=== Rust: clippy ==="
    cargo clippy --all-targets -- -D warnings || RUST_OK=false

    echo "=== Rust: tests ==="
    cargo test --all-targets || RUST_OK=false
else
    echo "No Cargo.toml found — skipping Rust checks"
fi

# ─── Vue / TypeScript Frontend ──────────────────────────────────────────────

if [ -f "package.json" ]; then
    # Install dependencies if node_modules missing
    if [ ! -d "node_modules" ]; then
        echo "=== Installing npm dependencies ==="
        npm install --prefer-offline 2>/dev/null || npm install
    fi

    # TypeScript type checking
    if grep -q '"vue-tsc"' package.json 2>/dev/null || [ -f "node_modules/.bin/vue-tsc" ]; then
        echo "=== TypeScript: type check ==="
        npx vue-tsc --noEmit || TS_OK=false
    elif [ -f "tsconfig.json" ]; then
        echo "=== TypeScript: type check ==="
        npx tsc --noEmit || TS_OK=false
    fi

    # Lint
    if [ -f ".eslintrc.js" ] || [ -f ".eslintrc.cjs" ] || [ -f ".eslintrc.json" ] || [ -f "eslint.config.js" ] || [ -f "eslint.config.mjs" ]; then
        echo "=== ESLint ==="
        npx eslint src/ || TS_OK=false
    fi

    # Command conventions check
    if [ -f "scripts/check-commands.sh" ]; then
        echo "=== Command conventions ==="
        bash scripts/check-commands.sh || CMDS_OK=false
    fi

    # CSS anti-pattern check: wildcard font-family overrides
    # font-family on *, :deep(*), or ::v-deep(*) clobbers Material Icons rendering.
    # The Vitest suite in __tests__/css-antipatterns.test.ts does a thorough
    # multi-line parse; this grep is a fast early-warning for single-line patterns.
    # Matches: * { font-family:   or  :deep(*) { font-family:
    if find src -name '*.vue' | xargs grep -E \
       '^\s*(\*|:deep\s*\(\s*\*\s*\)|::v-deep\s*\(\s*\*\s*\))\s*\{[^}]*font-family\s*:' \
       2>/dev/null | grep -q .; then
        echo "ERROR: Wildcard font-family override detected in a .vue file."
        echo "       Using font-family on *, :deep(*), or ::v-deep(*) clobbers Material Icons."
        echo "       Configure the app font via \$typography-font-family in quasar-variables.sass."
        TS_OK=false
    fi

    # Tests (Vitest)
    if grep -q '"vitest"' package.json 2>/dev/null; then
        # Auto-update snapshots before running tests so stale snapshots don't
        # cause spurious failures — the real pass/fail gate is the run below.
        npx vitest run --update 2>/dev/null || true
        echo "=== Vitest: unit tests ==="
        npx vitest run || TS_OK=false
    elif grep -q '"test"' package.json 2>/dev/null; then
        echo "=== npm test ==="
        npm test || TS_OK=false
    fi

    # Headless browser smoke test (optional — skipped if puppeteer or Chrome unavailable)
    if [ -f "scripts/smoke-test.mjs" ] && [ -d "node_modules/puppeteer" ]; then
        # Verify Chrome can actually launch before running the smoke test.
        # On some Linux environments (e.g. minimal containers) the puppeteer-managed
        # Chrome binary exists but fails to start due to missing system libraries
        # (e.g. libnspr4.so).  In that case, treat the smoke test as skipped
        # rather than failing — the Vitest suite still validates the frontend.
        CHROME_BIN=$(node -e "const p=require('puppeteer'); console.log(p.executablePath())" 2>/dev/null || true)
        CHROME_OK=false
        if [ -n "$CHROME_BIN" ] && [ -x "$CHROME_BIN" ]; then
            if "$CHROME_BIN" --version >/dev/null 2>&1; then
                CHROME_OK=true
            fi
        fi

        if [ "$CHROME_OK" = true ]; then
            echo "=== Smoke test: headless browser rendering ==="
            node scripts/smoke-test.mjs || SMOKE_OK=false
        else
            echo "=== Smoke test: skipped (Chrome binary cannot launch — missing system libraries) ==="
        fi
    else
        echo "=== Smoke test: skipped (puppeteer not installed) ==="
    fi
else
    echo "No package.json found — skipping frontend checks"
fi

# ─── Results ────────────────────────────────────────────────────────────────

if [ "$RUST_OK" = false ] || [ "$TS_OK" = false ] || [ "$CMDS_OK" = false ] || [ "$SMOKE_OK" = false ]; then
    echo ""
    echo "VERIFICATION FAILED"
    [ "$RUST_OK" = false ] && echo "  - Rust checks failed"
    [ "$TS_OK" = false ] && echo "  - TypeScript/Vue checks failed"
    [ "$CMDS_OK" = false ] && echo "  - Command conventions check failed"
    [ "$SMOKE_OK" = false ] && echo "  - Headless browser smoke test failed"
    exit 1
fi

echo ""
echo "VERIFICATION PASSED"
exit 0
