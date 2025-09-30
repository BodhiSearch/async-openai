# async-openai utoipa::ToSchema Annotation Implementation

## Overview

This documentation describes the automated addition of utoipa::ToSchema annotations to async-openai library types, enabling OpenAPI documentation generation.

## Problem Statement

async-openai types don't have the necessary `#[derive(utoipa::ToSchema)]` annotations required for OpenAPI schema generation when used with utoipa.

## Solution Architecture

### 1. Automated Annotation Script
- **Location**: `scripts/add_utoipa_annotations.py`
- **Purpose**: Automatically add utoipa::ToSchema annotations to types
- **Key Features**:
  - Type-level idempotency (won't re-add to types that already have it)
  - Adds as separate `#[derive()]` lines to maintain clarity
  - Excludes problematic types that contain non-ToSchema compatible fields
  - No import statements (uses fully qualified `utoipa::ToSchema`)

### 2. Rustfmt Configuration
- **Location**: `rustfmt.toml` (in async-openai root)
- **Configuration**:
  ```toml
  tab_spaces = 4
  merge_derives = false    # Prevents merging separate derive lines
  newline_style = "Unix"   # Ensures Unix-style line endings
  ```

### 3. Cargo Dependency
- **Location**: `async-openai/Cargo.toml`
- **Addition**:
  ```toml
  utoipa = { version = "5.3.1", features = ["preserve_order"] }
  ```

## Usage Instructions

### Running the Script

1. **Navigate to async-openai root**:
   ```bash
   cd async-openai
   ```

2. **Run the script**:
   ```bash
   python3 scripts/add_utoipa_annotations.py
   ```

3. **Verify changes**:
   ```bash
   # Check actual annotations added
   git diff -w | grep "^+.*utoipa::ToSchema" | wc -l

   # Test compilation
   cargo build -p async-openai
   ```

4. **Format code**:
   ```bash
   cargo fmt
   ```

### Re-running the Script

The script is idempotent and safe to re-run:
- Will skip types that already have annotations
- Won't duplicate annotations
- Preserves manual modifications

## Implementation Details

### Script Algorithm

1. **Pattern Matching**: Identifies struct/enum definitions with existing `#[derive()]` macros
2. **Idempotency Check**: Skips types that already have `utoipa::ToSchema`
3. **Type Exclusion**: Maintains a list of types to skip
4. **Annotation Addition**: Adds `#[derive(utoipa::ToSchema)]` as a separate line

### Files Modified

The script modifies files in `async-openai/src/types/`, typically adding 500+ annotations across 35+ files.

### Excluded Types

Types containing these fields are automatically excluded:
- `Bytes`, `ApiError`, `Arc<>`, `PathBuf`
- `InputSource`, `AudioInput`, `FileInput`
- `ImageInput`, `ResponseMetadata`
- Various other non-ToSchema compatible types

## Git Diff Behavior

### The "Whole File Changed" Issue

When running `git diff --stat`, some files appear to have hundreds of line changes despite only adding a few annotations. This is expected Git behavior when adding lines in the middle of files.

**How to verify actual changes**:
```bash
# Show actual changes ignoring whitespace
git diff -w src/types/audio.rs | grep "^+.*utoipa::ToSchema" | wc -l
```

## Maintenance

### Adding New Exclusions

If compilation fails due to new types with incompatible fields, edit the script:

```python
types_to_skip = [
    'CreateSpeechResponse',  # Contains Bytes
    'YourNewProblematicType',  # Add new exclusions here
    # ...
]
```

### Updating for New Versions

When updating async-openai:
1. Run the annotation script
2. Check for compilation errors
3. Add any new problematic types to exclusions
4. Re-run the script

## Troubleshooting

### Issue: cargo fmt merges derive lines
**Solution**: Ensure `rustfmt.toml` has `merge_derives = false`

### Issue: Large git diffs
**Solution**: This is expected. Use `git diff -w` to see actual changes

### Issue: Compilation errors after adding annotations
**Solution**: The type likely contains non-ToSchema compatible fields. Add it to the exclusion list.

### Issue: Script not finding files
**Solution**: Run from async-openai root directory, not from parent project

## Future Improvements

1. **Upstream Contribution**: Consider contributing utoipa support directly to async-openai
2. **Selective Annotation**: Add configuration to only annotate specific types needed
3. **Verification Script**: Add a script to verify all annotations compile correctly
4. **CI Integration**: Add to CI pipeline to ensure annotations stay current