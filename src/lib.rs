use pyo3::prelude::*;
use pyo3::types::{PyList};
use skim::prelude::*;

// Custom SkimItem implementation for holding a string
struct TextItem {
    text: String,
}

impl TextItem {
    fn new(text: &str) -> Self {
        TextItem {
            text: text.to_string(),
        }
    }
}

impl SkimItem for TextItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.text)
    }

    fn preview(&self, _: PreviewContext) -> ItemPreview {
        ItemPreview::Text(self.text.to_string())
    }

    // Add any other necessary implementations from the SkimItem trait
    fn display(&self, _: DisplayContext) -> AnsiString {
        // Convert &String to &str first
        let text_str: &str = self.text.as_str();
        AnsiString::from(text_str)
    }
}

/// Perform a fuzzy search on a list of strings
///
/// Args:
///     query: The search query
///     items: A list of strings to search
///
/// Returns:
///     A list of tuples containing (matched item, score)
#[pyfunction]
fn fuzzy_match(py: Python, query: &str, items: Vec<String>) -> PyResult<PyObject> {
    let results = PyList::empty(py);

    // Skip processing if the query is empty or there are no items
    if query.is_empty() || items.is_empty() {
        return Ok(results.into());
    }

    // Create a skim source from the items
    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .multi(true)
        .build()
        .unwrap();

    // Create a string of content for skim
    let content = items.join("\n");

    // Use the default item reader
    let item_reader = SkimItemReader::default();
    let items_source = item_reader.of_bufread(std::io::Cursor::new(content));

    // Run the search
    let result = Skim::run_with(&options, Some(items_source))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    // Process the results
    let mut scored_items = Vec::new();
    for item in &result {
        // Get the line as String
        let item_text = item.text().to_string();
        // Add to our results
        scored_items.push(item_text);
    }

    // Convert to Python list of tuples (item, score)
    let py_results = PyList::new(
        py,
        scored_items
            .iter()
            .collect::<Vec<_>>(),
    );

    Ok(py_results.into())
}

#[pymodule]
fn skym(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fuzzy_match, m)?)?;
    Ok(())
}
