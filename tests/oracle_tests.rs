/// Oracle tests: compare our NFA engine against the `regex` crate on real texts.
///
/// Setup:
///   1. Add `regex` to [dev-dependencies] in Cargo.toml:
///      regex = "1"
///
///   2. Download the test files (see download_benchdata.sh):
///      benches/alice.txt
///      benches/war_and_peace.txt
///      benches/bible.txt
///
///   3. Adjust the `use` imports below to match your crate structure.

use lin_reg::automaton::{Automaton, Match};

#[cfg(test)]
mod oracle_tests {
    use super::*;
    use regex::Regex;
    use std::fs::read_to_string;

    /// Runs our engine on a line, using substring-matching semantics.
    fn our_matches(automaton: &Automaton, line: &str) -> bool {
        let mut m = Match::new(automaton);
        m.recognizes(line)
    }

    /// Runs the oracle (regex crate) on a line.
    fn oracle_matches(re: &Regex, line: &str) -> bool {
        re.is_match(line)
    }

    /// Core comparison: run both engines on every line of a file,
    /// panic on the first disagreement with a useful error message.
    fn compare_on_file(file_path: &str, pattern: &str) {
        let content = read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Missing file: {file_path}. Run download_benchdata.sh first."));

        let automaton = Automaton::from_str(pattern)
            .unwrap_or_else(|| panic!("Our engine failed to compile: {pattern}"));

        // Wrap in explicit anchoring-free group so the oracle does substring matching
        // just like our engine. The regex crate does substring matching by default,
        // so we can pass the pattern directly.
        let oracle = Regex::new(pattern)
            .unwrap_or_else(|e| panic!("Oracle failed to compile '{pattern}': {e}"));

        let mut our_count = 0u32;
        let mut oracle_count = 0u32;

        for (line_num, line) in content.lines().enumerate() {
            let ours = our_matches(&automaton, line);
            let theirs = oracle_matches(&oracle, line);

            if ours { our_count += 1; }
            if theirs { oracle_count += 1; }

            if ours != theirs {
                panic!(
                    "Disagreement on file {file_path}, pattern '{pattern}', line {line_num}:\n\
                     Line:   {line:?}\n\
                     Ours:   {ours}\n\
                     Oracle: {theirs}"
                );
            }
        }

        // Sanity check: make sure we actually tested something.
        // Print match counts for visibility.
        eprintln!(
            "[OK] pattern '{pattern}' on {file_path}: {our_count} matches out of {} lines",
            content.lines().count()
        );
    }

    // ─────────────────────────────────────────
    //  Alice in Wonderland
    // ─────────────────────────────────────────

    #[test]
    fn alice_literal_alice() {
        compare_on_file("benches/alice.txt", "Alice");
    }

    #[test]
    fn alice_literal_queen() {
        compare_on_file("benches/alice.txt", "Queen");
    }

    #[test]
    fn alice_literal_rabbit() {
        compare_on_file("benches/alice.txt", "Rabbit");
    }

    #[test]
    fn alice_union_two() {
        compare_on_file("benches/alice.txt", "cat|hat");
    }

    #[test]
    fn alice_union_three() {
        compare_on_file("benches/alice.txt", "Alice|Queen|Rabbit");
    }

    #[test]
    fn alice_concat_the() {
        compare_on_file("benches/alice.txt", "the");
    }

    #[test]
    fn alice_star_simple() {
        // Matches any line containing one or more 'a's (or the empty match)
        compare_on_file("benches/alice.txt", "a*b");
    }

    #[test]
    fn alice_star_concat() {
        compare_on_file("benches/alice.txt", "(th)*e");
    }

    #[test]
    fn alice_star_union() {
        compare_on_file("benches/alice.txt", "(a|e)*d");
    }

    #[test]
    fn alice_textbook_pattern() {
        compare_on_file("benches/alice.txt", "(a|b)*abb");
    }

    #[test]
    fn alice_nested_star() {
        compare_on_file("benches/alice.txt", "(ab)*c");
    }

    // ─────────────────────────────────────────
    //  War and Peace
    // ─────────────────────────────────────────

    #[test]
    fn war_literal_prince() {
        compare_on_file("benches/war_and_peace.txt", "Prince");
    }

    #[test]
    fn war_literal_war() {
        compare_on_file("benches/war_and_peace.txt", "war");
    }

    #[test]
    fn war_union_war_peace() {
        compare_on_file("benches/war_and_peace.txt", "war|peace");
    }

    #[test]
    fn war_union_three() {
        compare_on_file("benches/war_and_peace.txt", "love|hate|war");
    }

    #[test]
    fn war_concat_long() {
        compare_on_file("benches/war_and_peace.txt", "Napoleon");
    }

    #[test]
    fn war_star_prefix() {
        compare_on_file("benches/war_and_peace.txt", "b(a|e)*d");
    }

    #[test]
    fn war_star_suffix() {
        compare_on_file("benches/war_and_peace.txt", "sp(e|o)*k");
    }

    #[test]
    fn war_complex_pattern() {
        compare_on_file("benches/war_and_peace.txt", "(a|o)*n(d|t)");
    }

    #[test]
    fn war_nested() {
        compare_on_file("benches/war_and_peace.txt", "((t|d)*he)*");
    }

    // ─────────────────────────────────────────
    //  King James Bible
    // ─────────────────────────────────────────

    #[test]
    fn bible_literal_god() {
        compare_on_file("benches/bible.txt", "God");
    }

    #[test]
    fn bible_literal_lord() {
        compare_on_file("benches/bible.txt", "Lord");
    }

    #[test]
    fn bible_union_god_lord() {
        compare_on_file("benches/bible.txt", "God|Lord");
    }

    #[test]
    fn bible_union_three() {
        compare_on_file("benches/bible.txt", "God|Lord|Jesus");
    }

    #[test]
    fn bible_concat_long() {
        compare_on_file("benches/bible.txt", "Jerusalem");
    }

    #[test]
    fn bible_star_simple() {
        compare_on_file("benches/bible.txt", "lo*k");
    }

    #[test]
    fn bible_star_union() {
        compare_on_file("benches/bible.txt", "(s|t)h(a|e)*ll");
    }

    #[test]
    fn bible_complex() {
        compare_on_file("benches/bible.txt", "th(e|a)(n|t)");
    }

    #[test]
    fn bible_star_of_concat() {
        compare_on_file("benches/bible.txt", "(an)*d");
    }

    #[test]
    fn bible_nested_star() {
        compare_on_file("benches/bible.txt", "((a|e)*n)*d");
    }

    // ─────────────────────────────────────────
    //  Edge case patterns on all files
    // ─────────────────────────────────────────

    #[test]
    fn all_files_single_char() {
        for file in &["benches/alice.txt", "benches/war_and_peace.txt", "benches/bible.txt"] {
            compare_on_file(file, "x");
        }
    }

    #[test]
    fn all_files_two_char_concat() {
        for file in &["benches/alice.txt", "benches/war_and_peace.txt", "benches/bible.txt"] {
            compare_on_file(file, "qu");
        }
    }

    #[test]
    fn all_files_star_of_union() {
        for file in &["benches/alice.txt", "benches/war_and_peace.txt", "benches/bible.txt"] {
            compare_on_file(file, "(a|b)*");
        }
    }

    #[test]
    fn all_files_complex_nested() {
        for file in &["benches/alice.txt", "benches/war_and_peace.txt", "benches/bible.txt"] {
            compare_on_file(file, "((a|b)*c(d|e)*)*f");
        }
    }
}