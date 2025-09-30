use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::OpenAIError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[derive(utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSyntax {
    Lark,
    #[default]
    Regex,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, Builder)]
#[derive(utoipa::ToSchema)]
#[builder(build_fn(error = "OpenAIError"))]
pub struct CustomGrammarFormatParam {
    /// The grammar definition.
    pub definition: String,
    /// The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: GrammarSyntax,
}
