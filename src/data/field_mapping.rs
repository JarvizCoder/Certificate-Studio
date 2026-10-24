use std::collections::HashMap;

/// Extract all dynamic placeholder keys enclosed in `{{ ... }}` within a template string.
pub fn extract_placeholders(text: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("{{") {
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("}}") {
            let key = after_start[..end].trim();
            if !key.is_empty() && !placeholders.iter().any(|p: &String| p.eq_ignore_ascii_case(key)) {
                placeholders.push(key.to_string());
            }
            rest = &after_start[end + 2..];
        } else {
            break;
        }
    }

    placeholders
}

/// Resolve a template string against a specific CSV record row.
///
/// If `template_text` contains `{{key}}`, replaces all instances.
/// If `template_text` contains no placeholders, but `field_key` is provided,
/// replaces the entire text with the value of that field key.
pub fn resolve_text_for_record(
    template_text: &str,
    field_key: Option<&str>,
    record: &HashMap<String, String>,
) -> String {
    // 1. If explicit placeholders {{...}} exist in text, substitute them
    if template_text.contains("{{") && template_text.contains("}}") {
        let mut result = template_text.to_string();
        for key in extract_placeholders(template_text) {
            let token = format!("{{{{{}}}}}", key);
            // Search case-insensitively in record
            let replacement = record
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(&key))
                .map(|(_, v)| v.as_str())
                .unwrap_or("");

            let rep_str = if replacement.is_empty() {
                format!("[Missing: {}]", key)
            } else {
                replacement.to_string()
            };

            // Replace exact pattern and spaced pattern
            result = result.replace(&token, &rep_str);
            result = result.replace(&format!("{{{{ {} }}}}", key), &rep_str);
        }
        result
    } else if let Some(key) = field_key {
        // 2. Direct field binding without explicit {{ }}
        if let Some((_, val)) = record.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)) {
            if !val.trim().is_empty() {
                return val.clone();
            }
        }
        template_text.to_string()
    } else {
        template_text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_placeholders() {
        let text = "Certifies that {{name}} participated in {{event}} with ID {{certificate_id}}.";
        let placeholders = extract_placeholders(text);
        assert_eq!(placeholders, vec!["name", "event", "certificate_id"]);
    }

    #[test]
    fn test_resolve_text_placeholders() {
        let mut record = HashMap::new();
        record.insert("name".to_string(), "Aarav Patil".to_string());
        record.insert("event".to_string(), "Code4Impact".to_string());

        let template = "Awarded to {{name}} for {{event}}.";
        let resolved = resolve_text_for_record(template, None, &record);
        assert_eq!(resolved, "Awarded to Aarav Patil for Code4Impact.");
    }

    #[test]
    fn test_resolve_text_direct_field_binding() {
        let mut record = HashMap::new();
        record.insert("name".to_string(), "Meera Shah".to_string());

        let template = "Default Name Placeholder";
        let resolved = resolve_text_for_record(template, Some("name"), &record);
        assert_eq!(resolved, "Meera Shah");
    }

    #[test]
    fn test_missing_placeholder_fallback() {
        let record = HashMap::new();
        let template = "Hello {{missing_field}}";
        let resolved = resolve_text_for_record(template, None, &record);
        assert_eq!(resolved, "Hello [Missing: missing_field]");
    }
}
