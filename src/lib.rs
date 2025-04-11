use pyo3::prelude::*;
use pyo3::types::{PyList};
use skim::prelude::*;

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
    // For empty query, we'll collect all items and return them without filtering

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

    // Return empty list if no items
    if item_strs.is_empty() {
        return Ok(PyList::empty(py).into());
    }

    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .query(Some(query.to_string()))
        .multi(true)
        .build()
        .unwrap();

    // Create a content string for skim
    let content = item_strs.join("\n");

    // Create source from our string content
    let item_reader = SkimItemReader::default();
    let source = item_reader.of_bufread(std::io::Cursor::new(content));

    // Run the fuzzy search
    let results = Skim::run_with(&options, Some(source))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    // Convert fuzzy search results to Python list
    let py_results = PyList::empty(py);
    for item in results {
        let item_text = item.text().to_string();
        py_results.append(item_text.into_py(py))?;
    }

    Ok(py_results.into())
}

#[pymodule]
fn skym(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fuzzy_match, m)?)?;
    Ok(())
}
