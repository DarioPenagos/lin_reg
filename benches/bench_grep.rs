use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs::read_to_string;

// Adjust this path to match your project's module structure.
// You may need to make the crate a lib+bin or move shared code to a library.
use lin_reg::automaton::{Automaton, Match};

/// Benchmarks a single regex against a full file, line by line (mimics grep).
fn bench_grep(
    c: &mut Criterion,
    file_path: &str,
    file_label: &str,
    regex: &str,
    regex_label: &str,
) {
    let content = read_to_string(file_path).unwrap_or_else(|_| {
        panic!("Missing benchmark file: {file_path}. See README for download instructions.")
    });
    let automaton =
        Automaton::from_str(regex).unwrap_or_else(|| panic!("Failed to compile regex: {regex}"));
    let lines: Vec<&str> = content.lines().collect();

    c.bench_with_input(
        BenchmarkId::new(format!("{file_label}/{regex_label}"), lines.len()),
        &lines,
        |b, lines| {
            b.iter(|| {
                let mut count = 0u32;
                for line in lines.iter() {
                    let mut matcher = Match::new(&automaton);
                    if matcher.recognizes(line) {
                        count += 1;
                    }
                }
                count
            })
        },
    );
}

/// Benchmarks compilation time for various regex patterns.
fn bench_compilation(c: &mut Criterion) {
    let patterns = vec![
        ("literal", "hello"),
        ("concat_short", "abcd"),
        ("concat_long", "abcdefghij"),
        ("union", "a|b|c|d"),
        ("star_simple", "a*"),
        ("star_concat", "(ab)*"),
        ("textbook", "(a|b)*abb"),
        ("nested", "((a|b)*c)*"),
        ("complex", "(a*b*|c(de)*)*"),
    ];

    let mut group = c.benchmark_group("compilation");
    for (label, pattern) in &patterns {
        group.bench_with_input(
            BenchmarkId::new(*label, pattern.len()),
            pattern,
            |b, pat| b.iter(|| Automaton::from_str(pat).unwrap()),
        );
    }
    group.finish();
}

/// Benchmarks matching against Alice in Wonderland.
fn bench_alice(c: &mut Criterion) {
    let patterns = vec![
        ("literal", "Alice"),
        ("union", "Alice|Queen|Rabbit"),
        ("star_prefix", "(a|b)*ing"),
        ("textbook", "(a|b)*abb"),
    ];

    for (label, regex) in patterns {
        bench_grep(c, "benches/alice.txt", "alice", regex, label);
    }
}

/// Benchmarks matching against War and Peace.
fn bench_war_and_peace(c: &mut Criterion) {
    let patterns = vec![
        ("literal", "Prince"),
        ("union", "war|peace|love"),
        ("star_prefix", "(a|b)*ing"),
        ("complex", "(a*b*|cd)*"),
    ];

    for (label, regex) in patterns {
        bench_grep(
            c,
            "benches/war_and_peace.txt",
            "war_and_peace",
            regex,
            label,
        );
    }
}

/// Benchmarks matching against the King James Bible.
fn bench_bible(c: &mut Criterion) {
    let patterns = vec![
        ("literal", "God"),
        ("union", "God|Lord|Jesus"),
        ("star_prefix", "(a|b)*tion"),
        ("complex", "((a|b)*c(d|e)*)*"),
    ];

    for (label, regex) in patterns {
        bench_grep(c, "benches/bible.txt", "bible", regex, label);
    }
}

/// Benchmarks pathological cases that stress the NFA.
fn bench_pathological(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological");

    // Worst case for backtracking engines: a?^n a^n
    // For NFA simulation this should be linear — verify that it is.
    for n in [10, 20, 30] {
        let regex_str = "a".repeat(n);
        let input = "a".repeat(n);

        let automaton = Automaton::from_str(&regex_str).unwrap();

        group.bench_with_input(BenchmarkId::new("linear_concat", n), &input, |b, input| {
            b.iter(|| {
                let mut matcher = Match::new(&automaton);
                matcher.recognizes(input)
            })
        });
    }

    // Star depth: (a*)* with long input — tests epsilon closure performance
    for n in [100, 500, 1000] {
        let input = "a".repeat(n);
        let automaton = Automaton::from_str("(a*)*").unwrap();

        group.bench_with_input(BenchmarkId::new("nested_star", n), &input, |b, input| {
            b.iter(|| {
                let mut matcher = Match::new(&automaton);
                matcher.recognizes(input)
            })
        });
    }

    // Many alternatives: a|b|c|d|e|f|g|h
    {
        let automaton = Automaton::from_str("a|b|c|d|e|f|g|h").unwrap();
        let input = "hhhhhhhhhh".repeat(100);

        group.bench_with_input(
            BenchmarkId::new("many_alternatives", input.len()),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut matcher = Match::new(&automaton);
                    matcher.recognizes(input)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compilation,
    bench_alice,
    bench_war_and_peace,
    bench_bible,
    bench_pathological,
);
criterion_main!(benches);
