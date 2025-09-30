use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Default, Debug, Deserialize, PartialEq)]
#[derive(utoipa::ToSchema)]
pub struct FunctionName {
    /// The name of the function to call.
    pub name: String,
}
