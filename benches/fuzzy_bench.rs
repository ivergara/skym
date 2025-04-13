use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
extern crate skym;
use skym::{bench_perform_non_interactive_match};

fn bench_non_interactive(c: &mut Criterion) {
    // Prepare test data
    let small_items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "orange".to_string(),
        "pineapple".to_string(),
        "grape".to_string(),
        "strawberry".to_string(),
        "blueberry".to_string(),
        "raspberry".to_string(),
        "blackberry".to_string(),
        "kiwi".to_string(),
        // Add more items to make the benchmark more realistic
        "mango".to_string(),
        "peach".to_string(),
        "plum".to_string(),
        "apricot".to_string(),
        "cherry".to_string(),
        "watermelon".to_string(),
        "cantaloupe".to_string(),
        "honeydew".to_string(),
        "papaya".to_string(),
        "guava".to_string(),
    ];

    c.bench_function("fuzzy_match_common_query", |b| {
        b.iter(|| {
            bench_perform_non_interactive_match(black_box("ap"), black_box(small_items.clone()))
        })
    });

    c.bench_function("fuzzy_match_rare_query", |b| {
        b.iter(|| {
            bench_perform_non_interactive_match(black_box("zx"), black_box(small_items.clone()))
        })
    });

    // Create datasets of different sizes to measure scaling behavior
    let sizes = [10, 100, 1000];
    let mut group = c.benchmark_group("scaling_behavior");

    for size in sizes {
        let items: Vec<String> = (0..size)
            .map(|i| format!("item_{:04}", i))
            .collect();

        group.bench_with_input(BenchmarkId::new("items", size), &items, |b, items| {
            b.iter(|| {
                bench_perform_non_interactive_match(black_box("item_5"), black_box(items.clone()))
            })
        });
    }

    group.finish();

    // Test with very long strings to see impact of string length
    let long_items: Vec<String> = (0..20)
        .map(|i| {
            let mut s = format!("item_{:04}_", i);
            // Add a lot of text to make the string long
            for _ in 0..100 {
                s.push_str("padding_text_");
            }
            s
        })
        .collect();

    c.bench_function("fuzzy_match_long_strings", |b| {
        b.iter(|| {
            bench_perform_non_interactive_match(black_box("item_1"), black_box(long_items.clone()))
        })
    });
}

criterion_group!(benches, bench_non_interactive);
criterion_main!(benches);
