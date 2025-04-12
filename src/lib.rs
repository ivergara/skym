use pyo3::prelude::*;
use pyo3::types::{PyList};
use skim::prelude::*;

/// Perform the actual fuzzy matching logic
///
/// This function separates the core matching logic from the Python binding,
/// making it easier to test and maintain.
///
/// Args:
///     query: The search query
///     items: A vector of strings to search
///
/// Returns:
///     A vector of matched strings
pub fn perform_fuzzy_match(query: &str, items: Vec<String>) -> Vec<String> {
    // Return empty vector if no items
    if items.is_empty() {
        return Vec::new();
    }

    // Configure the skim options
    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .query(Some(query.to_string()))
        .multi(true)
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

/// Perform a fuzzy search on an iterable of strings
///
/// Args:
///     query: The search query
///     items: An iterable of strings to search
///
/// Returns:
///     A list of matched items
#[pyfunction]
fn fuzzy_match(py: Python, query: &str, items: PyObject) -> PyResult<PyObject> {
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
    let matched_items = perform_fuzzy_match(query, item_strs);

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
