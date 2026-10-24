use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maps CSV headers to certificate template placeholders.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Placeholder tag -> CSV column name
    pub mapping: HashMap<String, String>,
}

