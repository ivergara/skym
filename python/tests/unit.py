import pytest
from skym import fuzzy_match

# Common test fixtures
@pytest.fixture
def basic_items():
    return ["apple", "banana", "cherry"]

@pytest.fixture
def extended_items():
    return ["apple", "application", "apology", "appetite"]

@pytest.fixture
def product_items():
    return [
        "application development",
        "apple",
        "apply",
        "apartment",
        "banana with apple"
    ]

@pytest.fixture
def case_sensitive_items():
    return ["Apple", "Banana", "Cherry"]

@pytest.fixture
def complex_items():
    return ["apple pie", "banana split", "cherry tart"]

# Fixtures for different iterable types
@pytest.fixture
def tuple_items():
    return ("apple", "banana", "cherry")

@pytest.fixture
def set_items():
    return {"apple", "banana", "cherry"}

@pytest.fixture
def generator_items():
    return (item for item in ["apple", "banana", "cherry"])

@pytest.fixture
def custom_iterable():
    class CustomIterable:
        def __iter__(self):
            return iter(["apple", "banana", "cherry"])
    return CustomIterable()

# Basic functionality tests
def test_empty_input():
    """Test that an empty input list returns an empty result."""
    result = fuzzy_match("test", [], interactive=False)
    assert result == []

def test_exact_match(basic_items):
    """Test that exact matches are returned."""
    result = fuzzy_match("apple", basic_items, interactive=False)
    assert "apple" in result
    assert result[0] == "apple"  # Exact match should be first

def test_case_insensitive(case_sensitive_items):
    """Test that matching is case-insensitive."""
    result = fuzzy_match("apple", case_sensitive_items, interactive=False)
    assert "Apple" in result

def test_substring_match(complex_items):
    """Test that substrings are matched."""
    result = fuzzy_match("apple", complex_items, interactive=False)
    assert "apple pie" in result

def test_fuzzy_match(extended_items):
    """Test that fuzzy matching works."""
    result = fuzzy_match("aple", extended_items, interactive=False)
    # "apple" should be matched and ranked high
    assert "apple" in result
    assert result.index("apple") < len(result) / 2  # Should be in the first half

def test_no_match(basic_items):
    """Test that queries with no matches return empty list."""
    result = fuzzy_match("xylophone", basic_items, interactive=False)
    assert result == []

# Parametrized tests for different iterable types
@pytest.mark.parametrize("items_fixture", [
    "basic_items",        # List
    "tuple_items",        # Tuple
    "set_items",          # Set
    "generator_items",    # Generator
    "custom_iterable"     # Custom iterable
])
def test_different_iterables(items_fixture, request):
    """Test that different iterable types work correctly."""
    items = request.getfixturevalue(items_fixture)
    result = fuzzy_match("a", items, interactive=False)
    assert "apple" in result

# Parametrized tests for various query and expected result patterns
@pytest.mark.parametrize("query,expected_first", [
    ("apple", "apple"),           # Exact match
    ("app", "apple"),             # Prefix match
    ("aple", "apple"),            # Fuzzy match with deletion
    ("a", "apple"),               # Single character match
])
def test_query_patterns(extended_items, query, expected_first):
    """Test different query patterns and their expected top results."""
    result = fuzzy_match(query, extended_items, interactive=False)
    assert result[0] == expected_first

# Edge cases
@pytest.mark.skip(reason="Skipping edge cases for now")
@pytest.mark.parametrize("query,items,expected", [
    ("", ["apple", "banana"], []),          # Empty query
    ("apple", ["APPLE"], ["APPLE"]),        # All uppercase
    ("a p p l e", ["apple"], ["apple"]),    # Spaces in query
    ("日本語", ["日本語", "中文"], ["日本語"]),  # Non-Latin characters
    ("\t\napple", ["apple"], ["apple"]),    # Whitespace in query
])
def test_edge_cases(query, items, expected):
    """Test edge cases for fuzzy matching."""
    result = fuzzy_match(query, items, interactive=False)
    assert result == expected
