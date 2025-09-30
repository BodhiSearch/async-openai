#!/usr/bin/env python3
"""
Script to add utoipa::ToSchema annotations to async-openai types.

This script processes all .rs files in async-openai/async-openai/src/types/
and adds separate #[derive(utoipa::ToSchema)] lines for structs and enums.

Features:
- Adds separate derive lines instead of modifying existing ones
- Idempotent: skips types that already have utoipa::ToSchema annotations
- No import statements added (uses fully qualified paths)
"""

import re
from pathlib import Path
import sys
import os


def add_utoipa_to_file(file_path):
    """Add separate #[derive(utoipa::ToSchema)] lines to structs and enums"""
    try:
        content = file_path.read_text()
        original_content = content

        # Types to skip (contain types that don't implement ToSchema)
        types_to_skip = [
            'CreateSpeechResponse',  # Contains Bytes
            'AssistantStreamEvent',  # Contains ApiError
            'ImageResponse',         # Contains Arc<Image>
            'Image'                  # Contains Arc<String>
        ]

        # Pattern to match derive macros followed by struct/enum definitions
        # This captures:
        # - Optional non-derive attributes like #[builder(...)] (group 1)
        # - One or more consecutive #[derive(...)] lines (group 2)
        # - Optional attributes like #[serde(...)] (group 3)
        # - Optional doc comments (group 4)
        # - The actual struct/enum definition (group 5, name in group 6)
        pattern = r'((?:#\[[^d][^\]]*\]\s*|#\[d(?!erive\()[^\]]*\]\s*)*)((?:#\[derive\([^)]+\)\]\s*)+)((?:#\[[^\]]+\]\s*)*)((?:///[^\n]*\n\s*)*)(pub (?:struct|enum) (\w+))'

        def add_separate_derive(match):
            pre_derive_attrs = match.group(1) if match.group(1) else ""
            existing_derives = match.group(2).rstrip()
            other_attributes = match.group(3) if match.group(3) else ""
            doc_comments = match.group(4) if match.group(4) else ""
            type_definition = match.group(5)
            type_name = match.group(6)

            # Type-level idempotency: Skip if this specific type already has utoipa::ToSchema
            if 'utoipa::ToSchema' in existing_derives:
                return match.group(0)

            # Skip problematic types
            if type_name in types_to_skip:
                return match.group(0)

            # Skip types that contain fields with non-ToSchema types
            # Look ahead to check the content of the struct/enum
            rest_of_content = content[match.end():]
            struct_content = ""
            brace_count = 0
            in_struct = False

            for char in rest_of_content:
                if char == '{':
                    brace_count += 1
                    in_struct = True
                elif char == '}':
                    brace_count -= 1
                    if brace_count == 0 and in_struct:
                        break

                if in_struct:
                    struct_content += char

            # Skip types with problematic field types
            if any(problematic_type in struct_content for problematic_type in [
                'Bytes', 'ApiError', 'Arc<', 'PathBuf',
                'InputSource', 'WebSearchPreview', 'AudioInput',
                'FileInput', 'HostedToolType', 'ToolDefinition', 'ImageInput',
                'ResponseMetadata'
            ]):
                return match.group(0)

            # Add our derive line after existing derives but before other attributes
            if other_attributes:
                return f'{pre_derive_attrs}{existing_derives}\n#[derive(utoipa::ToSchema)]\n{other_attributes}{doc_comments}{type_definition}'
            else:
                return f'{pre_derive_attrs}{existing_derives}\n#[derive(utoipa::ToSchema)]\n{doc_comments}{type_definition}'

        # Apply pattern to add separate derive lines
        modified = re.sub(pattern, add_separate_derive, content, flags=re.MULTILINE)

        return modified, modified != original_content

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return None, False


def process_types_directory():
    """Process all files in async-openai/src/types/"""
    types_dir = Path('async-openai/src/types')

    if not types_dir.exists():
        print(f"Error: Directory {types_dir} not found!")
        print("Make sure you're running this from the async-openai repository root.")
        return False

    files_modified = 0
    total_files = 0

    print(f"Processing files in {types_dir}...")

    # Process all .rs files except mod.rs
    for rust_file in types_dir.glob('*.rs'):
        if rust_file.name == 'mod.rs':
            continue

        total_files += 1
        print(f"Processing {rust_file.name}...")

        result, was_modified = add_utoipa_to_file(rust_file)

        if result is None:
            print(f"❌ Failed to process {rust_file.name}")
            continue

        if was_modified:
            rust_file.write_text(result)
            files_modified += 1
            print(f"✅ Modified {rust_file.name}")
        else:
            print(f"⏭️  No changes needed for {rust_file.name}")

    # Also check subdirectories
    for subdir in types_dir.iterdir():
        if subdir.is_dir():
            print(f"\nProcessing subdirectory: {subdir.name}")
            for rust_file in subdir.glob('**/*.rs'):
                if rust_file.name == 'mod.rs':
                    continue

                total_files += 1
                print(f"Processing {rust_file.relative_to(types_dir)}...")

                result, was_modified = add_utoipa_to_file(rust_file)

                if result is None:
                    print(f"❌ Failed to process {rust_file.name}")
                    continue

                if was_modified:
                    rust_file.write_text(result)
                    files_modified += 1
                    print(f"✅ Modified {rust_file.relative_to(types_dir)}")
                else:
                    print(f"⏭️  No changes needed for {rust_file.relative_to(types_dir)}")

    print(f"\n📊 Summary:")
    print(f"   Total files processed: {total_files}")
    print(f"   Files modified: {files_modified}")
    print(f"   Files unchanged: {total_files - files_modified}")

    return True


def main():
    """Main entry point"""
    print("🚀 async-openai utoipa Annotation Script")
    print("=" * 50)

    # Ensure we're in the right directory
    if not Path('async-openai/Cargo.toml').exists():
        print("❌ Error: async-openai/Cargo.toml not found!")
        print("Make sure you're running this from the async-openai repository root.")
        print("Example: cd async-openai && python3 scripts/add_utoipa_annotations.py")
        sys.exit(1)

    success = process_types_directory()

    if success:
        print("\n✅ Annotation script completed successfully!")
        print("\n📝 Next steps:")
        print("   1. Test compilation: cargo build -p async-openai")
        print("   2. Check git diff to review changes: git diff")
        print("   3. Run cargo fmt if needed: cargo fmt")
    else:
        print("\n❌ Script completed with errors.")
        sys.exit(1)


if __name__ == '__main__':
    main()