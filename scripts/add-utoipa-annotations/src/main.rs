#!/usr/bin/env rust-script
//! Script to add utoipa::ToSchema annotations to async-openai types.
//!
//! This script processes all .rs files in async-openai/src/types/
//! and adds separate #[derive(utoipa::ToSchema)] lines for structs and enums.
//!
//! Features:
//! - Uses syn crate for proper Rust AST parsing
//! - Adds separate derive lines instead of modifying existing ones
//! - Idempotent: skips types that already have utoipa::ToSchema annotations
//! - No import statements added (uses fully qualified paths)

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Attribute, DeriveInput, Fields, Item, Meta, Type};
use walkdir::WalkDir;

/// Types to skip (contain types that don't implement ToSchema)
fn get_skip_list() -> HashSet<&'static str> {
    vec![
        "CreateSpeechResponse", // Contains Bytes
        "AssistantStreamEvent", // Contains ApiError
        "ImageResponse",        // Contains Arc<Image>
        "Image",                // Contains Arc<String>
    ]
    .into_iter()
    .collect()
}

/// Problematic field types that indicate a type shouldn't get ToSchema
fn get_problematic_types() -> Vec<&'static str> {
    vec![
        "Bytes",
        "ApiError",
        "Arc<",
        "PathBuf",
        "InputSource",
        "WebSearchPreview",
        "AudioInput",
        "FileInput",
        "HostedToolType",
        "ToolDefinition",
        "ImageInput",
        "ResponseMetadata",
    ]
}

/// Visitor to check if a type contains problematic field types
struct FieldTypeChecker {
    has_problematic_type: bool,
    problematic_types: Vec<String>,
}

impl FieldTypeChecker {
    fn new() -> Self {
        Self {
            has_problematic_type: false,
            problematic_types: get_problematic_types().iter().map(|s| s.to_string()).collect(),
        }
    }

    fn check_fields(fields: &Fields) -> bool {
        let mut checker = Self::new();
        checker.visit_fields(fields);
        checker.has_problematic_type
    }
}

impl<'ast> Visit<'ast> for FieldTypeChecker {
    fn visit_type(&mut self, ty: &'ast Type) {
        // Convert type to string, removing all whitespace for reliable matching
        let type_string = quote::quote!(#ty).to_string().replace(" ", "");
        for problematic in &self.problematic_types {
            let problematic_no_space = problematic.replace(" ", "");
            if type_string.contains(&problematic_no_space) {
                self.has_problematic_type = true;
                return;
            }
        }
        syn::visit::visit_type(self, ty);
    }
}

/// Check if derive attributes already contain utoipa::ToSchema
fn has_utoipa_derive(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("derive") {
            if let Meta::List(ref meta_list) = attr.meta {
                let tokens = meta_list.tokens.to_string();
                if tokens.contains("utoipa :: ToSchema") || tokens.contains("utoipa::ToSchema") {
                    return true;
                }
            }
        }
    }
    false
}

/// Find the position to insert the new derive attribute
/// Returns the index after the last derive attribute
fn find_derive_insert_position(attrs: &[Attribute]) -> usize {
    let mut last_derive_pos = 0;
    for (i, attr) in attrs.iter().enumerate() {
        if attr.path().is_ident("derive") {
            last_derive_pos = i + 1;
        }
    }
    last_derive_pos
}

/// Visitor to add utoipa::ToSchema derives to structs and enums
struct UtoipaAnnotator {
    skip_list: HashSet<&'static str>,
    modified: bool,
}

impl UtoipaAnnotator {
    fn new() -> Self {
        Self {
            skip_list: get_skip_list(),
            modified: false,
        }
    }

    fn should_add_utoipa(&self, derive_input: &DeriveInput) -> bool {
        let type_name = derive_input.ident.to_string();

        // Skip if in skip list
        if self.skip_list.contains(type_name.as_str()) {
            return false;
        }

        // Skip if already has utoipa::ToSchema
        if has_utoipa_derive(&derive_input.attrs) {
            return false;
        }

        // Check for problematic field types
        match &derive_input.data {
            syn::Data::Struct(data_struct) => {
                if FieldTypeChecker::check_fields(&data_struct.fields) {
                    return false;
                }
            }
            syn::Data::Enum(data_enum) => {
                for variant in &data_enum.variants {
                    if FieldTypeChecker::check_fields(&variant.fields) {
                        return false;
                    }
                }
            }
            syn::Data::Union(_) => return false,
        }

        true
    }

    fn add_utoipa_derive(&mut self, derive_input: &mut DeriveInput) {
        if !self.should_add_utoipa(derive_input) {
            return;
        }

        // Create the new derive attribute
        let new_derive: Attribute = syn::parse_quote! {
            #[derive(utoipa::ToSchema)]
        };

        // Find position to insert (after last derive)
        let insert_pos = find_derive_insert_position(&derive_input.attrs);

        // Insert the new derive attribute
        derive_input.attrs.insert(insert_pos, new_derive);
        self.modified = true;
    }
}

impl VisitMut for UtoipaAnnotator {
    fn visit_item_mut(&mut self, item: &mut Item) {
        match item {
            Item::Struct(item_struct) => {
                let mut derive_input = DeriveInput {
                    attrs: item_struct.attrs.clone(),
                    vis: item_struct.vis.clone(),
                    ident: item_struct.ident.clone(),
                    generics: item_struct.generics.clone(),
                    data: syn::Data::Struct(syn::DataStruct {
                        struct_token: item_struct.struct_token,
                        fields: item_struct.fields.clone(),
                        semi_token: item_struct.semi_token,
                    }),
                };

                self.add_utoipa_derive(&mut derive_input);
                item_struct.attrs = derive_input.attrs;
            }
            Item::Enum(item_enum) => {
                let mut derive_input = DeriveInput {
                    attrs: item_enum.attrs.clone(),
                    vis: item_enum.vis.clone(),
                    ident: item_enum.ident.clone(),
                    generics: item_enum.generics.clone(),
                    data: syn::Data::Enum(syn::DataEnum {
                        enum_token: item_enum.enum_token,
                        brace_token: item_enum.brace_token,
                        variants: item_enum.variants.clone(),
                    }),
                };

                self.add_utoipa_derive(&mut derive_input);
                item_enum.attrs = derive_input.attrs;
            }
            _ => {}
        }

        syn::visit_mut::visit_item_mut(self, item);
    }
}

/// Process a single Rust file
fn process_file(path: &Path) -> Result<bool> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut syntax_tree: syn::File = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut annotator = UtoipaAnnotator::new();
    annotator.visit_file_mut(&mut syntax_tree);

    if annotator.modified {
        let formatted = prettyplease::unparse(&syntax_tree);
        std::fs::write(path, formatted)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Process all files in the types directory
fn process_types_directory() -> Result<()> {
    let types_dir = PathBuf::from("async-openai/src/types");

    if !types_dir.exists() {
        anyhow::bail!(
            "Error: Directory {} not found!\nMake sure you're running this from the async-openai repository root.",
            types_dir.display()
        );
    }

    let mut files_modified = 0;
    let mut total_files = 0;

    println!("Processing files in {}...", types_dir.display());

    for entry in WalkDir::new(&types_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .filter(|e| e.path().file_name().map_or(false, |name| name != "mod.rs"))
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(&types_dir).unwrap_or(path);

        total_files += 1;
        print!("Processing {}...", relative_path.display());

        match process_file(path) {
            Ok(true) => {
                files_modified += 1;
                println!(" ✅ Modified");
            }
            Ok(false) => {
                println!(" ⏭️  No changes needed");
            }
            Err(e) => {
                println!(" ❌ Failed: {}", e);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("   Total files processed: {}", total_files);
    println!("   Files modified: {}", files_modified);
    println!("   Files unchanged: {}", total_files - files_modified);

    Ok(())
}

fn main() -> Result<()> {
    println!("🚀 async-openai utoipa Annotation Script (Rust version)");
    println!("{}", "=".repeat(60));

    // Ensure we're in the right directory
    if !Path::new("async-openai/Cargo.toml").exists() {
        anyhow::bail!(
            "❌ Error: async-openai/Cargo.toml not found!\n\
             Make sure you're running this from the async-openai repository root.\n\
             Example: cd async-openai && cargo run --manifest-path scripts/add-utoipa-annotations/Cargo.toml"
        );
    }

    process_types_directory()?;

    println!("\n✅ Annotation script completed successfully!");
    println!("\n📝 Next steps:");
    println!("   1. Test compilation: cargo build -p async-openai");
    println!("   2. Check git diff to review changes: git diff");
    println!("   3. Run cargo fmt if needed: cargo fmt");

    Ok(())
}