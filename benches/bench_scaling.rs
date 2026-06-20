//! Scaling benchmarks for the lin_reg NFA engine.
//!
//! Place at `benches/bench_scaling.rs` and add to Cargo.toml:
//!   [[bench]]
//!   name = "bench_scaling"
//!   harness = false
//! Run with `cargo bench --bench bench_scaling`.
//! Plots land in target/criterion/<group>/report/ (open target/criterion/report/index.html).

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

// IMPORT PATH: I couldn't read your src/ layout (GitHub blocks automated access
// to the file tree), and `Automaton` isn't re-exported at the crate root. Adjust
// this to wherever `Automaton` and `Match` actually live, e.g.
//   use lin_reg::automaton::{Automaton, Match};
use lin_reg::{automaton::Automaton, automaton::Match};

/// Drive the `Match` runner across the whole input.
///
/// We OR `is_accepting()` across positions instead of returning on first match,
/// so the benchmark always performs the full sweep (the real O(m*n) cost) rather
/// than stopping early. This also makes the measurement independent of whether
/// `step` currently runs anchored or unanchored (state-0 re-injection): either
/// way every character is stepped. With the non-matching workloads below, the
/// out-of-alphabet branch of `step` never fires.
#[inline]
fn full_scan(nfa: &Automaton, input: &str) -> bool {
    let mut m = Match::new(nfa);
    let mut accepted = m.is_accepting(); // position 0 (handles nullable / empty input)
    for c in input.chars() {
        if m.step(c).is_none() {
            break; // out-of-alphabet; unreachable for the all-`a` inputs used here
        }
        accepted |= m.is_accepting();
    }
    accepted
}

fn compile(pattern: &str) -> Automaton {
    Automaton::from_str(pattern).expect("benchmark pattern must be valid")
}

// All-`a` input: every char is in-alphabet (so each step does real transition
// work), but there is no `b`, so the `b`-requiring patterns below never match.
// That forces a full scan and lets us measure the whole O(m*n) cost.
fn input_all_a(len: usize) -> String {
    "a".repeat(len)
}

/// Fixed-size pattern requiring a `b`: never matches an all-`a` input.
const FIXED_PATTERN: &str = "(a|b)*b";

/// n-axis pattern: k ε-chained `a*` blocks then a required `b`. The `a*` blocks
/// are all simultaneously active while scanning `a`s, so the active set grows
/// ~linearly in k (= NFA size) -- this is what actually exercises the O(n)
/// factor. Never matches an all-`a` input.
fn sized_pattern(k: usize) -> String {
    let mut p = "a*".repeat(k);
    p.push('b');
    p
}

/// (1) Match time vs input length m, pattern fixed.  Expect: linear in m.
fn bench_input_length(c: &mut Criterion) {
    let nfa = compile(FIXED_PATTERN);
    let mut g = c.benchmark_group("match_vs_input_len");
    for &m in &[1_000usize, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000] {
        let input = input_all_a(m);
        g.throughput(Throughput::Bytes(m as u64)); // ns/byte reporting
        g.bench_with_input(BenchmarkId::from_parameter(m), &input, |b, input| {
            b.iter(|| black_box(full_scan(&nfa, black_box(input))));
        });
    }
    g.finish();
}

/// (2) Match time vs NFA size n (~k), input fixed.  Expect: linear in n.
fn bench_nfa_size(c: &mut Criterion) {
    let input = input_all_a(20_000);
    let mut g = c.benchmark_group("match_vs_nfa_size");
    for &k in &[1usize, 2, 4, 8, 16, 32, 64] {
        let nfa = compile(&sized_pattern(k)); // compiled once, outside the timed loop
        g.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, _| {
            b.iter(|| black_box(full_scan(&nfa, black_box(&input))));
        });
    }
    g.finish();
}

/// (3) Construction cost vs pattern size (from_str = parse + Thompson build).
fn bench_compile(c: &mut Criterion) {
    let mut g = c.benchmark_group("compile_vs_pattern_size");
    for &k in &[1usize, 2, 4, 8, 16, 32, 64] {
        let pat = sized_pattern(k);
        g.bench_with_input(BenchmarkId::from_parameter(k), &pat, |b, pat| {
            b.iter(|| black_box(compile(black_box(pat))));
        });
    }
    g.finish();
}

criterion_group!(benches, bench_input_length, bench_nfa_size, bench_compile);
criterion_main!(benches);
