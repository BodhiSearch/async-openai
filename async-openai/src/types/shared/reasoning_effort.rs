use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq, Default)]
#[derive(utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    Xhigh,
}
