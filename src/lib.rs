use pyo3::prelude::*;
use pyo3::types::{PyList};
use skim::prelude::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

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
        .unwrap();

    // Create a content string for skim
    let content = items.join("\n");

    // Create source from our string content
    let item_reader = SkimItemReader::default();
    let source = item_reader.of_bufread(std::io::Cursor::new(content));

    // Run the fuzzy search
    let results = Skim::run_with(&options, Some(source))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    // Convert skim results to string vector
    results.iter()
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

    // Score each item and collect results
    let mut scored_items: Vec<(i64, String)> = items.iter()
        .filter_map(|item| {
            // Score the item, filter out non-matches
            matcher.fuzzy_match(item, query)
                .map(|score| (score, item.clone()))
        })
        .collect();

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
fn fuzzy_match(py: Python, query: &str, items: PyObject, interactive: Option<bool>) -> PyResult<PyObject> {
    // Convert items to an iterator
    let items = items.as_ref(py);
    let iter = items.iter()?;

    // Collect the strings from the iterator
    let mut item_strs = Vec::new();
    for item_result in iter {
        let item = item_result?;
        let item_str = item.extract::<String>()?;
        item_strs.push(item_str);
    }

    // Use our helper function to perform the actual matching
    // Default to non-interactive mode if not specified
    let is_interactive = interactive.unwrap_or(false);
    let matched_items = perform_fuzzy_match(query, item_strs, is_interactive);

    // Convert results to Python list
    let py_results = PyList::empty(py);
    for item in matched_items {
        py_results.append(item.into_py(py))?;
    }

    Ok(py_results.into())
}

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
