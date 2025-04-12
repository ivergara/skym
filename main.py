from skym import fuzzy_match


def example_string_match():
    """Example of searching in a list of strings"""
    items = ["apple", "banana", "orange", "pineapple", "apricot", "grape", "avocado"]

    # Simple search
    results = fuzzy_match("", items)
    print("String search results:")
    print(results)


if __name__ == "__main__":
    example_string_match()
