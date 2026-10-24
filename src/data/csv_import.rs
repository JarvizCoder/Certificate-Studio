use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Represents an imported CSV tabular dataset.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CsvDataset {
    pub source_path: Option<PathBuf>,
    pub filename: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl CsvDataset {
    /// Parse CSV data from any reader.
    pub fn parse_from_reader<R: Read>(reader: R, filename: impl Into<String>) -> Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(reader);

        let headers = rdr
            .headers()
            .context("Failed to read CSV headers")?
            .iter()
            .map(|h| h.trim().to_string())
            .collect::<Vec<_>>();

        let mut rows = Vec::new();
        for result in rdr.records() {
            let record = result.context("Failed to parse CSV record")?;
            let mut row = Vec::with_capacity(headers.len());
            for field in record.iter() {
                row.push(field.trim().to_string());
            }
            // Pad row if shorter than headers
            while row.len() < headers.len() {
                row.push(String::new());
            }
            rows.push(row);
        }

        Ok(Self {
            source_path: None,
            filename: filename.into(),
            headers,
            rows,
        })
    }

    /// Parse a CSV file from disk.
    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .with_context(|| format!("Failed to open CSV file at {:?}", path_ref))?;

        let filename = path_ref
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("dataset.csv")
            .to_string();

        let mut dataset = Self::parse_from_reader(file, filename)?;
        dataset.source_path = Some(path_ref.to_path_buf());
        Ok(dataset)
    }

    pub fn total_records(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Retrieve a row as a key-value map of header name -> field value.
    pub fn row_as_map(&self, row_idx: usize) -> Option<HashMap<String, String>> {
        let row = self.rows.get(row_idx)?;
        let mut map = HashMap::with_capacity(self.headers.len());
        for (i, header) in self.headers.iter().enumerate() {
            let val = row.get(i).cloned().unwrap_or_default();
            map.insert(header.clone(), val);
        }
        Some(map)
    }

    /// Validate all records against required bound placeholder keys.
    /// Returns a list of (row_idx_1_based, missing_keys).
    pub fn validate_required_keys(&self, required_keys: &[String]) -> Vec<(usize, Vec<String>)> {
        let mut failures = Vec::new();

        for (idx, row) in self.rows.iter().enumerate() {
            let mut row_map = HashMap::new();
            for (col_idx, header) in self.headers.iter().enumerate() {
                let val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                row_map.insert(header.to_lowercase(), val);
            }

            let mut missing = Vec::new();
            for req in required_keys {
                let req_lower = req.to_lowercase();
                match row_map.get(&req_lower) {
                    Some(val) if val.trim().is_empty() => {
                        missing.push(req.clone());
                    }
                    None => {
                        missing.push(req.clone());
                    }
                    _ => {}
                }
            }

            if !missing.is_empty() {
                failures.push((idx + 1, missing));
            }
        }

        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing_and_mapping() {
        let csv_data = "name,event,certificate_id\nAarav Patil,Code4Impact,CI-001\nMeera Shah,Code4Impact,CI-002\n";
        let dataset = CsvDataset::parse_from_reader(csv_data.as_bytes(), "test.csv").unwrap();

        assert_eq!(dataset.headers, vec!["name", "event", "certificate_id"]);
        assert_eq!(dataset.total_records(), 2);

        let row0 = dataset.row_as_map(0).unwrap();
        assert_eq!(row0.get("name").unwrap(), "Aarav Patil");
        assert_eq!(row0.get("certificate_id").unwrap(), "CI-001");

        let row1 = dataset.row_as_map(1).unwrap();
        assert_eq!(row1.get("name").unwrap(), "Meera Shah");
    }

    #[test]
    fn test_csv_validation() {
        let csv_data = "name,event\nAarav Patil,Code4Impact\n,Hackathon\n";
        let dataset = CsvDataset::parse_from_reader(csv_data.as_bytes(), "test.csv").unwrap();

        let required = vec!["name".to_string(), "event".to_string()];
        let failures = dataset.validate_required_keys(&required);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, 2); // Row 2 has empty name
        assert_eq!(failures[0].1, vec!["name".to_string()]);
    }
}
