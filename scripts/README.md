# Scripts

## add-utoipa-annotations

Adds `#[derive(utoipa::ToSchema)]` and `#[schema(value_type = ...)]` annotations
to all structs and enums in `async-openai/src/types/`.

**Run from the async-openai repository root:**

```sh
cargo run --manifest-path scripts/add-utoipa-annotations/Cargo.toml
```

### What it does

- Adds a separate `#[derive(utoipa::ToSchema)]` line after existing `#[derive(...)]` blocks
- Adds `#[schema(value_type = T)]` on fields/variants whose type is:
  - `serde_json::Value` → `Object`
  - `Option<serde_json::Value>` → `Option<Object>`
  - `Vec<serde_json::Value>` → `Vec<Object>`
  - `HashMap<_, serde_json::Value>` / `BTreeMap<_, serde_json::Value>` → `Object`
- Handles hardcoded special cases for recursive type cycles that cannot be
  auto-detected from the field type (e.g. `CompoundFilter.filters: Vec<Filter>`)
- Idempotent — safe to run multiple times
- Preserves all inline comments and original formatting (text insertion, not AST rewrite)

### After running

```sh
cargo fmt
cargo run --package xtask openapi   # regenerate openapi.json
# OR
make build.ts-client                # regenerate openapi.json + TypeScript types

git diff                            # review changes
```

### If generation fails with a stack overflow

A stack overflow during `cargo xtask openapi` or `make build.ts-client` means
utoipa hit a recursive type cycle that the script didn't annotate.

1. Identify the offending type from the stack trace
2. Add a new entry to `get_special_case_insertions()` in
   `scripts/add-utoipa-annotations/src/main.rs`
3. Re-run the script and regenerate
