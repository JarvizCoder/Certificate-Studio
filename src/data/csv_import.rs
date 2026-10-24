use serde::{Deserialize, Serialize};

/// Represents imported tabular data from CSV.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CsvDataset {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

