use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence, PyString};
use skim::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;

/// Perform the actual fuzzy matching logic
///
/// This function separates the core matching logic from the Python binding,
/// making it easier to test and maintain.
///
/// Args:
///     query: The search query
///     items: A slice of strings to search
///     interactive: Whether to run skim in interactive mode
///
/// Returns:
///     A vector of matched strings or PyErr if something fails
fn perform_fuzzy_match<'a>(
    query: &str,
    items: &'a [String],
    interactive: bool,
) -> PyResult<Vec<&'a String>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

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
///     items: A slice of strings to search
///
/// Returns:
///     A vector of matched strings or PyErr if something fails
fn perform_interactive_match<'a>(query: &str, items: &'a [String]) -> PyResult<Vec<&'a String>> {
    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .query(Some(query.to_string()))
        .multi(true)
        .interactive(true)
        .build()
        .map_err(|err| PyRuntimeError::new_err(format!("Failed to build skim options: {}", err)))?;

    let mut selected_indices = Vec::with_capacity(2);

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

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = bounded(skim_items.len());

    for item in skim_items {
        if let Err(err) = tx.send(item) {
            return Err(PyRuntimeError::new_err(format!(
                "Failed to send item to skim channel: {}",
                err
            )));
        }
    }
    drop(tx);

    if let Some(out) = Skim::run_with(&options, Some(rx)) {
        for item in out.selected_items {
            if let Some(string_item) = item.as_any().downcast_ref::<StringItem>() {
                selected_indices.push(string_item.index);
            }
        }
    }

    Ok(selected_indices
        .iter()
        .filter_map(|&idx| items.get(idx))
        .collect())
}

/// Perform non-interactive fuzzy matching using fuzzy-matcher
///
/// Args:
///     query: The search query
///     items: A slice of strings to search
///
/// Returns:
///     A vector of matched strings or PyErr if something fails
fn perform_non_interactive_match<'a>(
    query: &str,
    items: &'a [String],
) -> PyResult<Vec<&'a String>> {
    // Create a SkimMatcherV2 (same algorithm used by skim)
    let matcher = SkimMatcherV2::default();

    let mut scored_indices: Vec<(i64, usize)> = Vec::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        if let Some(score) = matcher.fuzzy_match(item, query) {
            scored_indices.push((score, index));
        }
    }

    // Sort by score (descending)
    scored_indices.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(scored_indices
        .into_iter()
        .filter_map(|(_, index)| items.get(index))
        .collect())
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
///
/// Raises:
///     TypeError: If None is found in the items
///     ValueError: If any non-string item is found in the items
///     RuntimeError: If there's an error in the underlying fuzzy matching system
#[pyfunction]
fn fuzzy_match(
    py: Python,
    query: &str,
    items: &PyAny,
    interactive: Option<bool>,
) -> PyResult<PyObject> {
    let iter = items.iter()?;

    // Get the length of the iterator if it's a sequence
    let approx_len = if let Ok(seq) = items.downcast::<PySequence>() {
        seq.len().unwrap_or(10).min(1000) // Cap at 1000 to avoid excessive allocation
    } else {
        10 // Default capacity if we can't determine length
    };

    let mut item_strs = Vec::with_capacity(approx_len);

    for item_result in iter {
        let item = item_result?;

        if item.is_none() {
            return Err(PyTypeError::new_err(
                "'NoneType' object cannot be converted to string",
            ));
        }

        let str_item = if let Ok(py_str) = item.downcast::<PyString>() {
            // Fast path for Python strings
            py_str.to_str()?.to_owned()
        } else {
            // Fallback for other types
            match item.extract::<String>() {
                Ok(s) => s,
                Err(_) => {
                    // Get the type name of the problematic item
                    let type_name = item.get_type().name().unwrap_or("Unknown");
                    return Err(PyValueError::new_err(format!(
                        "'{}' object cannot be converted to string",
                        type_name
                    )));
                }
            }
        };

        item_strs.push(str_item);
    }

    let is_interactive = interactive.unwrap_or(false);
    let matched_items = perform_fuzzy_match(query, &item_strs, is_interactive)?;

    let py_results = PyList::new(py, matched_items.iter().map(|&s| s.clone()));

    Ok(py_results.into())
}

/// A Python module implemented in Rust performing (non) interactive fuzzy matching of a string over an iterable of strings.
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

#[doc(hidden)]
pub fn bench_perform_fuzzy_match<'a>(
    query: &str,
    items: &'a [String],
    interactive: bool,
) -> PyResult<Vec<&'a String>> {
    perform_fuzzy_match(query, items, interactive)
}

#[doc(hidden)]
pub fn bench_perform_non_interactive_match<'a>(
    query: &str,
    items: &'a [String],
) -> PyResult<Vec<&'a String>> {
    perform_non_interactive_match(query, items)
}
