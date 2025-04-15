use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence};
use pyo3::exceptions::PyValueError;
use skim::prelude::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::sync::Arc;

/// Perform the actual fuzzy matching logic
///
/// This function separates the core matching logic from the Python binding,
/// making it easier to test and maintain.
///
/// Args:
///     query: The search query
///     items: A vector of strings to search
///     interactive: Whether to run skim in interactive mode
///
/// Returns:
///     A vector of matched strings
fn perform_fuzzy_match(query: &str, items: Vec<String>, interactive: bool) -> Vec<String> {
    // Return early for empty input
    if items.is_empty() {
        return Vec::new();
    }

    // Use a match expression for clearer intent
    match interactive {
        true => perform_interactive_match(query, items),
        false => perform_non_interactive_match(query, items),
    }
}

struct StringItem {
    text: String,
    index: usize,
}

impl SkimItem for StringItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.text)
    }

    fn output(&self) -> Cow<str> {
        self.text()
    }
}

/// Perform interactive fuzzy matching using skim
///
/// Args:
///     query: The search query
///     items: A vector of strings to search
///
/// Returns:
///     A vector of matched strings
fn perform_interactive_match(query: &str, items: Vec<String>) -> Vec<String> {
    // Configure the skim options
    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .query(Some(query.to_string()))
        .multi(true)
        .interactive(true)
        .build()
        .expect("Failed to build skim options");

    let skim_items: Vec<Arc<dyn SkimItem>> = items
        .iter()
        .enumerate()
        .map(|(i, text)| {
            Arc::new(StringItem {
                text: text.clone(),
                index: i,
            }) as Arc<dyn SkimItem>
        })
        .collect();

    // Create a Receiver for the items
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = bounded(skim_items.len());

    // Send all items to the channel
    for item in skim_items {
        tx.send(item).unwrap();
    }
    drop(tx); // Close the channel

    // Run the fuzzy search with the receiver
    let selected_items = Skim::run_with(&options, Some(rx))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    // Convert skim results to string vector
    selected_items.iter()
        .map(|item| item.text().to_string())
        .collect()
}

/// Perform non-interactive fuzzy matching using fuzzy-matcher
///
/// Args:
///     query: The search query
///     items: A vector of strings to search
///
/// Returns:
///     A vector of matched strings
fn perform_non_interactive_match(query: &str, items: Vec<String>) -> Vec<String> {
    // Create a SkimMatcherV2 (same algorithm used by skim)
    let matcher = SkimMatcherV2::default();

    // Preallocate vector with capacity equal to items (worst case all match)
    let mut scored_items: Vec<(i64, String)> = Vec::with_capacity(items.len());

    // Score each item and collect results
    for item in &items {
        if let Some(score) = matcher.fuzzy_match(item, query) {
            scored_items.push((score, item.clone()));
        }
    }

    // Sort by score (descending)
    scored_items.sort_by(|a, b| b.0.cmp(&a.0));

    // Extract just the strings
    scored_items.into_iter()
        .map(|(_, item)| item)
        .collect()
}

/// Perform a fuzzy search on an iterable of strings
///
/// Args:
///     query: The search query
///     items: An iterable of strings to search
///     interactive: Whether to run in interactive mode (default: False).
///                  When True, shows a UI for selection. When False, returns matches non-interactively.
///
/// Returns:
///     A list of matched items
#[pyfunction]
fn fuzzy_match(py: Python, query: &str, items: &PyAny, interactive: Option<bool>) -> PyResult<PyObject> {
    // Convert items to an iterator
    let iter = items.iter()?;

    // Get the length of the iterator if it's a sequence
    let approx_len = if let Ok(seq) = items.downcast::<PySequence>() {
        seq.len().unwrap_or(10).min(1000) // Cap at 1000 to avoid excessive allocation
    } else {
        10 // Default capacity if we can't determine length
    };

    // Collect with capacity hint
    let mut item_strs = Vec::with_capacity(approx_len);
    for item_result in iter {
        let item = item_result?;
        let str_item = item.extract::<String>()
            .map_err(|_| {
                let type_name = item.get_type().name().unwrap_or("Unknown");
                PyValueError::new_err(
                    format!("Expected a string item, got object of type: {}", type_name)
                )
            })?;
        item_strs.push(str_item);
    }
    // .collect();
    // let item_strs = item_strs?;

    // Use our helper function to perform the actual matching
    // Default to non-interactive mode if not specified
    let is_interactive = interactive.unwrap_or(false);
    let matched_items = perform_fuzzy_match(query, item_strs, is_interactive);

    // Create Python list directly from the matched string items
    // This shouldn't fail since we have a Vec<String> that can be converted to Python strings
    let py_results = PyList::new(py, matched_items);

    Ok(py_results.into())
}

/// A Python module implemented in Rust performing (non) interactive fuzzy matching of a string iver an iterable of strings.
#[pymodule]
fn skym(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fuzzy_match, m)?)?;
    Ok(())
}

// ----------------------------------------------------------------------
// BENCHMARK WRAPPER FUNCTIONS
//
// These functions are directly exported for benchmarking and testing.
// ----------------------------------------------------------------------

/// Wrapper function for benchmarking perform_fuzzy_match
/// This function is exported for benchmarks but not intended for general use
#[doc(hidden)]
pub fn bench_perform_fuzzy_match(query: &str, items: Vec<String>, interactive: bool) -> Vec<String> {
    perform_fuzzy_match(query, items, interactive)
}

/// Wrapper function for benchmarking perform_non_interactive_match
/// This function is exported for benchmarks but not intended for general use
#[doc(hidden)]
pub fn bench_perform_non_interactive_match(query: &str, items: Vec<String>) -> Vec<String> {
    perform_non_interactive_match(query, items)
}
