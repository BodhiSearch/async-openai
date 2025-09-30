# utoipa Annotations for async-openai

This async-openai fork includes tooling to automatically add `utoipa::ToSchema` annotations for OpenAPI documentation generation.

## Quick Start

```bash
# From async-openai root directory
python3 scripts/add_utoipa_annotations.py
```

## What It Does

The script adds `#[derive(utoipa::ToSchema)]` annotations to structs and enums in `async-openai/src/types/`, enabling them to be used with utoipa for OpenAPI schema generation.

## Key Features

- **Idempotent**: Safe to run multiple times
- **Smart exclusions**: Skips types that can't implement ToSchema
- **Preserves formatting**: Works with cargo fmt

## Configuration

Ensure `rustfmt.toml` contains:
```toml
merge_derives = false
```

And add to `async-openai/Cargo.toml`:
```toml
utoipa = { version = "5.3.1", features = ["preserve_order"] }
```

## Documentation

See `docs/utoipa-annotations/` for detailed documentation.

## Files

- `scripts/add_utoipa_annotations.py` - The annotation script
- `docs/utoipa-annotations/README.md` - Full documentation
- `rustfmt.toml` - Formatting configuration