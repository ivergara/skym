import random
import string

import pytest
from skym import fuzzy_match


# Helper function to generate test data
def generate_random_strings(count, min_length=3, max_length=20):
    """Generate a list of random strings."""
    strings = []
    for _ in range(count):
        length = random.randint(min_length, max_length)
        random_string = "".join(random.choices(string.ascii_lowercase, k=length))
        strings.append(random_string)
    return strings


# Dataset fixtures with different sizes
@pytest.fixture(scope="module")
def small_dataset():
    random.seed(42)  # For reproducibility
    return generate_random_strings(100)


@pytest.fixture(scope="module")
def medium_dataset():
    random.seed(42)  # For reproducibility
    return generate_random_strings(1000)


@pytest.fixture(scope="module")
def large_dataset():
    random.seed(42)  # For reproducibility
    return generate_random_strings(10000)


# Specialized dataset fixtures
@pytest.fixture(scope="module")
def common_words():
    words = [
        "the",
        "be",
        "to",
        "of",
        "and",
        "a",
        "in",
        "that",
        "have",
        "I",
        "it",
        "for",
        "not",
        "on",
        "with",
        "he",
        "as",
        "you",
        "do",
        "at",
        "this",
        "but",
        "his",
        "by",
        "from",
        "they",
        "we",
        "say",
        "her",
        "she",
        "or",
        "an",
        "will",
        "my",
        "one",
        "all",
        "would",
        "there",
        "their",
        "what",
        "so",
        "up",
        "out",
        "if",
        "about",
        "who",
        "get",
        "which",
        "go",
        "me",
        "when",
        "make",
        "can",
        "like",
        "time",
        "no",
        "just",
        "him",
        "know",
        "take",
        "people",
        "into",
        "year",
        "your",
        "good",
        "some",
        "could",
        "them",
        "see",
        "other",
        "than",
        "then",
        "now",
        "look",
        "only",
        "come",
        "its",
        "over",
        "think",
        "also",
        "back",
        "after",
        "use",
        "two",
        "how",
        "our",
        "work",
        "first",
        "well",
        "way",
        "even",
        "new",
        "want",
        "because",
        "any",
        "these",
        "give",
        "day",
        "most",
        "us",
    ]
    return words


@pytest.fixture(scope="module")
def structured_data():
    # Create a list of product names
    products = [
        "iPhone 13 Pro 256GB",
        "Samsung Galaxy S22 Ultra",
        "MacBook Pro 16-inch M1 Max",
        "Dell XPS 15 Laptop",
        "Sony WH-1000XM4 Headphones",
        "Bose QuietComfort 45",
        "iPad Air 10.9-inch",
        "Microsoft Surface Pro 8",
        "Canon EOS R5 Camera",
        "Sony A7 IV Camera",
        "LG C1 OLED 65-inch TV",
        "Samsung QN90A Neo QLED TV",
        "Dyson V11 Vacuum",
        "Roomba i7+ Robot Vacuum",
        "Ninja Foodi Pressure Cooker",
        "Instant Pot Duo Plus",
        "KitchenAid Stand Mixer",
        "Vitamix 5200 Blender",
        "Herman Miller Aeron Chair",
        "Steelcase Gesture Chair",
    ]

    # Add variations
    variations = []
    for product in products:
        variations.append(f"{product} - New")
        variations.append(f"{product} - Refurbished")
        variations.append(f"2023 Model {product}")

    return products + variations


# Query fixtures
@pytest.fixture
def exact_query(dataset):
    """Get an exact query from the dataset."""
    random.seed(42)
    return random.choice(dataset)


@pytest.fixture
def fuzzy_query(dataset):
    """Get a fuzzy query by modifying an item from the dataset."""
    random.seed(42)
    item = random.choice(dataset)
    if len(item) > 3:
        pos = random.randint(1, len(item) - 2)
        return item[:pos] + item[pos + 1 :]
    return item


@pytest.fixture
def no_match_query():
    """Get a query that will definitely not match."""
    return "xyzabcdefghijklmnopqrstuvw"


# Fixture for different iterable types
@pytest.fixture
def different_iterables(medium_dataset):
    return {
        "list": medium_dataset,
        "tuple": tuple(medium_dataset),
        "set": set(medium_dataset[:100]),  # Limit size for set
        "generator": (x for x in medium_dataset),
    }


# Parametrized benchmarks for dataset sizes
@pytest.mark.parametrize(
    "dataset_name", ["small_dataset", "medium_dataset", "large_dataset"]
)
def test_bench_exact_match(benchmark, request, dataset_name):
    """Benchmark exact match on different dataset sizes."""
    dataset = request.getfixturevalue(dataset_name)
    random.seed(42)
    query = random.choice(dataset)
    benchmark(fuzzy_match, query, dataset, False)


@pytest.mark.parametrize(
    "dataset_name", ["small_dataset", "medium_dataset", "large_dataset"]
)
def test_bench_fuzzy_match(benchmark, request, dataset_name):
    """Benchmark fuzzy match on different dataset sizes."""
    dataset = request.getfixturevalue(dataset_name)
    random.seed(42)
    item = random.choice(dataset)
    if len(item) > 3:
        pos = random.randint(1, len(item) - 2)
        query = item[:pos] + item[pos + 1 :]
    else:
        query = item
    benchmark(fuzzy_match, query, dataset, False)


# Benchmarks for specialized datasets
@pytest.mark.parametrize("query", ["th", "woud"])
def test_bench_common_words(benchmark, common_words, query):
    """Benchmark with common English words."""
    benchmark(fuzzy_match, query, common_words, False)


@pytest.mark.parametrize(
    "query", ["iphone", "mackbook"]
)  # "mackbook" is a typo for "MacBook"
def test_bench_structured_data(benchmark, structured_data, query):
    """Benchmark with structured product data."""
    benchmark(fuzzy_match, query, structured_data, False)


# Benchmark different iterable types
def test_bench_different_iterables(benchmark, different_iterables):
    """Benchmark with different iterable types."""

    def run_with_different_iterables():
        for iterable_type, iterable in different_iterables.items():
            fuzzy_match("a", iterable, False)

    benchmark(run_with_different_iterables)


# Benchmark edge cases
def test_bench_no_matches(benchmark, medium_dataset, no_match_query):
    """Benchmark with a query that has no matches."""
    benchmark(fuzzy_match, no_match_query, medium_dataset, False)
