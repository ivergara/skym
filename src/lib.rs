use pyo3::prelude::*;
use std::sync::Arc;
use std::borrow::Cow;

// Import skim components
use skim::prelude::*;

/// A simple string item for skim
struct SimpleItem {
    text: String,
}

impl SkimItem for SimpleItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.text)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::Text(self.text.to_string())
    }
}

/// Simple fuzzy matching implementation
fn simple_fuzzy_match(text: &str, pattern: &str) -> Option<i64> {
    // Basic implementation of fuzzy matching
    // If pattern is empty, it matches with score 0
    if pattern.is_empty() {
        return Some(0);
    }

    let mut pattern_chars = pattern.chars().peekable();
    let mut score: i64 = 0;
    let mut last_matched_pos: Option<usize> = None;

    // Check if all pattern characters appear in order in the text
    for (i, ch) in text.chars().enumerate() {
        if let Some(&pattern_ch) = pattern_chars.peek() {
            if ch.to_lowercase().next() == pattern_ch.to_lowercase().next() {
                pattern_chars.next();

                // Compute score: adjacent matches get higher scores
                let position_bonus = if let Some(last_pos) = last_matched_pos {
                    if i == last_pos + 1 {
                        10 // Adjacent match bonus
                    } else {
                        0
                    }
                } else {
                    0
                };

                score += 1 + position_bonus;
                last_matched_pos = Some(i);

                // If we've matched all pattern chars, we're done
                if pattern_chars.peek().is_none() {
                    return Some(score);
                }
            }
        }
    }

    // If we get here with no more pattern chars, it's a match
    if pattern_chars.peek().is_none() {
        Some(score)
    } else {
        None
    }
}

/// A basic function to perform fuzzy matching with skim
#[pyfunction]
fn fuzzy_match(query: &str, choices: Vec<String>) -> PyResult<Vec<(usize, String, i64)>> {
    // Create items from the choices
    let items: Vec<SimpleItem> = choices
        .iter()
        .map(|choice| {
            SimpleItem {
                text: choice.to_string(),
            }
        })
        .collect();

    // Perform matching
    let mut results = Vec::new();
    for (index, item) in items.iter().enumerate() {
        // Use our own fuzzy matching function
        if let Some(score) = simple_fuzzy_match(&item.text, query) {
            // Skip very low scores
            if score > 0 {
                results.push((index, item.text.to_string(), score));
            }
        }
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(results)
}

/// A Python module implemented in Rust.
#[pymodule]
fn skym(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fuzzy_match, py)?)?;
    Ok(())
}
