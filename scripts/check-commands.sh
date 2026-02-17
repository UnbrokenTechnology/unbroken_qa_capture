#!/usr/bin/env bash
# check-commands.sh
# Verifies that all Tauri commands invoked by the frontend are registered in the backend,
# and (stretch goal) that every backend-registered command has a #[tauri::command] fn.
#
# Usage: bash scripts/check-commands.sh
# Run from the repository root.

set -euo pipefail

FRONTEND_FILE="src/api/tauri.ts"
BACKEND_FILE="src-tauri/src/lib.rs"
RUST_SRC_DIR="src-tauri/src"

PASS=0
FAIL=0

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

error()   { echo -e "${RED}[FAIL]${NC} $*" >&2; }
success() { echo -e "${GREEN}[PASS]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }

# ── Sanity-check that the required files exist ────────────────────────────────
for f in "$FRONTEND_FILE" "$BACKEND_FILE"; do
    if [[ ! -f "$f" ]]; then
        error "Required file not found: $f"
        error "Run this script from the repository root."
        exit 1
    fi
done

# ── 1. Extract frontend invoke() command names ────────────────────────────────
# Matches:  invoke('command_name' ...  and  invoke<T>('command_name' ...
# Extracts the first quoted string argument (single or double quotes).
frontend_commands=$(
    grep -oE "invoke(<[^>]+>)?\s*\(\s*['\"][a-z_]+['\"]" "$FRONTEND_FILE" \
    | sed -E "s/.*['\"]([a-z_]+)['\"].*/\1/" \
    | sort -u
)

if [[ -z "$frontend_commands" ]]; then
    error "No invoke() calls found in $FRONTEND_FILE – pattern may need updating."
    exit 1
fi

frontend_count=$(echo "$frontend_commands" | wc -l | tr -d ' ')
echo "Found $frontend_count frontend command(s) in $FRONTEND_FILE"

# ── 2. Extract backend generate_handler![] command names ─────────────────────
# The macro call spans multiple lines; we locate the macro block and pull
# every bare identifier (one per line, possibly followed by a comma).
#
# Strategy:
#   a) Find the line number of "generate_handler![" in lib.rs.
#   b) From that line onward, grab lines until we hit the closing "])"
#      (the end of the macro invocation).
#   c) Extract identifier tokens from those lines.

handler_start=$(grep -n "generate_handler!\[" "$BACKEND_FILE" | tail -1 | cut -d: -f1)

if [[ -z "$handler_start" ]]; then
    error "generate_handler![ not found in $BACKEND_FILE"
    exit 1
fi

# Read from handler_start to the line containing the closing "])"
# We stop at the first line that matches the closing bracket pattern.
backend_commands=$(
    awk -v start="$handler_start" '
        NR > start {
            # Stop before the closing "])" line (do not print it)
            if (/\]\)/) exit
            print
        }
    ' "$BACKEND_FILE" \
    | grep -oE '\b[a-z][a-z0-9_]+\b' \
    | sort -u
)

if [[ -z "$backend_commands" ]]; then
    error "No commands extracted from generate_handler![] in $BACKEND_FILE"
    exit 1
fi

backend_count=$(echo "$backend_commands" | wc -l | tr -d ' ')
echo "Found $backend_count backend command(s) in $BACKEND_FILE"
echo ""

# ── 3. Compare: frontend commands missing from backend ───────────────────────
echo "=== Check 1: Frontend commands present in backend ==="

# comm -23 requires both inputs sorted; we already sorted them above.
missing_in_backend=$(comm -23 \
    <(echo "$frontend_commands") \
    <(echo "$backend_commands") \
)

if [[ -z "$missing_in_backend" ]]; then
    success "All frontend commands are registered in the backend."
    PASS=$((PASS + 1))
else
    error "The following frontend command(s) are NOT registered in the backend:"
    while IFS= read -r cmd; do
        echo "    - $cmd" >&2
    done <<< "$missing_in_backend"
    FAIL=$((FAIL + 1))
fi

echo ""

# ── 4. Informational: backend commands not called by frontend ─────────────────
echo "=== Info: Backend commands not called by frontend (informational) ==="

extra_in_backend=$(comm -13 \
    <(echo "$frontend_commands") \
    <(echo "$backend_commands") \
)

if [[ -z "$extra_in_backend" ]]; then
    echo "  (none – every backend command is called by the frontend)"
else
    warn "The following backend command(s) are NOT called by the frontend (may be unused or called elsewhere):"
    while IFS= read -r cmd; do
        echo "    - $cmd"
    done <<< "$extra_in_backend"
fi

echo ""

# ── 5. STRETCH GOAL: Every generate_handler![] entry has a #[tauri::command] fn ──
echo "=== Check 2: Every backend-registered command has a #[tauri::command] fn ==="

# Collect all function names that are decorated with #[tauri::command].
# We look at the line immediately following the attribute.
# Pattern: "#[tauri::command]" on one line, then "fn <name>" on the next
# (allowing for async, pub, etc.).
tauri_command_fns=$(
    grep -rn --include="*.rs" -A1 "#\[tauri::command\]" "$RUST_SRC_DIR" \
    | grep -oE '\bfn\s+[a-z][a-z0-9_]+' \
    | sed 's/^fn[[:space:]]*//' \
    | sort -u
)

if [[ -z "$tauri_command_fns" ]]; then
    warn "No #[tauri::command] functions found in $RUST_SRC_DIR – stretch goal skipped."
else
    fn_count=$(echo "$tauri_command_fns" | wc -l | tr -d ' ')
    echo "Found $fn_count #[tauri::command] function(s) across $RUST_SRC_DIR/**/*.rs"

    missing_attribute=$(comm -23 \
        <(echo "$backend_commands") \
        <(echo "$tauri_command_fns") \
    )

    if [[ -z "$missing_attribute" ]]; then
        success "Every generate_handler![] entry has a matching #[tauri::command] fn."
        PASS=$((PASS + 1))
    else
        error "The following generate_handler![] command(s) have NO matching #[tauri::command] fn:"
        while IFS= read -r cmd; do
            echo "    - $cmd" >&2
        done <<< "$missing_attribute"
        FAIL=$((FAIL + 1))
    fi
fi

echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "=== Summary ==="
echo "  Checks passed : $PASS"
echo "  Checks failed : $FAIL"
echo ""

if [[ "$FAIL" -gt 0 ]]; then
    error "$FAIL check(s) failed. See details above."
    exit 1
else
    success "All checks passed."
    exit 0
fi
