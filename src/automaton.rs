use crate::bool_alg::*;
use crate::parse::RegexNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Automaton {
    states: usize,
    transition: HashMap<char, BoolMat>,
    final_states: BoolVec,
    epsilon_closure: BoolMat,
}

pub struct Match<'a> {
    automaton: &'a Automaton,
    active: BoolVec,
}

impl Automaton {
    fn literal(c: char) -> Self {
        Automaton {
            states: 2,
            transition: HashMap::from([(c, BoolMat::from_coord(vec![(0, 1)], (2, 2)).unwrap())]),
            final_states: BoolVec::from_indices(vec![1], 2).unwrap(),
            epsilon_closure: BoolMat::identity(2),
        }
    }

    fn kleene_star(mut self) -> Self {
        self.states += 1;
        for (_, mat) in self.transition.iter_mut() {
            mat.kleene_shift()
        }

        self.epsilon_closure.kleene_shift();
        self.epsilon_closure.insert(0, 1);

        self.final_states.kleene_shift();

        for x in self.final_states.val_indx.iter() {
            self.epsilon_closure.insert(*x, 0);
        }

        self.epsilon_closure.close_epsilon();
        self.final_states.set_indices(vec![0]);
        self
    }

    fn concat(a: Self, b: Self) -> Self {
        let a_states = a.states;
        let b_states = b.states;

        let mut transition = b.transition;
        let mut epsilon_closure = b.epsilon_closure;

        for &k in a.transition.keys() {
            transition
                .entry(k)
                .or_insert(BoolMat::zeros(b_states, b_states));
        }

        for v in transition.values_mut() {
            v.shift_by(a_states);
        }

        epsilon_closure.shift_by(a_states);

        for (k, mat) in a.transition {
            for (m, wind) in mat.row_ptr.windows(2).enumerate() {
                for &n in &mat.col_indx[wind[0]..wind[1]] {
                    transition.get_mut(&k).unwrap().insert(m, n);
                }
            }
        }

        for (m, wind) in a.epsilon_closure.row_ptr.windows(2).enumerate() {
            for &n in &a.epsilon_closure.col_indx[wind[0]..wind[1]] {
                epsilon_closure.insert(m, n);
            }
        }

        for state in a.final_states.val_indx {
            epsilon_closure.insert(state, a_states);
        }

        let final_states = BoolVec::from_indices(
            b.final_states
                .val_indx
                .iter()
                .map(|x| x + a_states)
                .collect(),
            a_states + b_states,
        )
        .unwrap();

        epsilon_closure.close_epsilon();
        Automaton {
            states: a_states + b_states,
            transition,
            final_states,
            epsilon_closure,
        }
    }

    fn union(a: Self, b: Self) -> Self {
        let a_states = a.states;
        let b_states = b.states;

        let mut transition = b.transition;
        let mut epsilon_closure = b.epsilon_closure;

        for &k in a.transition.keys() {
            transition
                .entry(k)
                .or_insert(BoolMat::zeros(b_states, b_states));
        }

        for v in transition.values_mut() {
            v.shift_by(a_states + 1);
        }

        epsilon_closure.shift_by(a_states + 1);

        for (k, mat) in a.transition {
            for (m, wind) in mat.row_ptr.windows(2).enumerate() {
                for &n in &mat.col_indx[wind[0]..wind[1]] {
                    transition.get_mut(&k).unwrap().insert(m + 1, n + 1);
                }
            }
        }

        for (m, wind) in a.epsilon_closure.row_ptr.windows(2).enumerate() {
            for &n in &a.epsilon_closure.col_indx[wind[0]..wind[1]] {
                epsilon_closure.insert(m + 1, n + 1);
            }
        }

        epsilon_closure.insert(0, 1);
        epsilon_closure.insert(0, a_states + 1);

        epsilon_closure.close_epsilon();

        let new_final = a
            .final_states
            .val_indx
            .iter()
            .map(|x| x + 1)
            .chain(b.final_states.val_indx.iter().map(|x| x + a_states + 1))
            .collect();

        let final_states = BoolVec::from_indices(new_final, a_states + b_states + 1).unwrap();

        Automaton {
            states: a_states + b_states + 1,
            transition,
            final_states,
            epsilon_closure,
        }
    }

    fn empty() -> Self {
        Automaton {
            states: 1,
            transition: HashMap::new(),
            final_states: BoolVec::from_indices(vec![0], 1).unwrap(),
            epsilon_closure: BoolMat::identity(1),
        }
    }

    fn from_regex_node(node: RegexNode) -> Self {
        match node.clone() {
            RegexNode::Concat(a, b) => {
                Self::concat(Self::from_regex_node(*a), Self::from_regex_node(*b))
            }
            RegexNode::Union(a, b) => {
                Self::union(Self::from_regex_node(*a), Self::from_regex_node(*b))
            }
            RegexNode::KleeneStar(a) => Self::kleene_star(Self::from_regex_node(*a)),
            RegexNode::Literal(c) => Self::literal(c),
            RegexNode::Empty => Self::empty(),
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Some(Self::from_regex_node(RegexNode::parse(value)?))
    }
}

impl<'a> Match<'a> {
    pub fn new(automaton: &'a Automaton) -> Self {
        Match {
            automaton: automaton,
            active: (&BoolVec::from_indices(vec![0], automaton.states).unwrap()
                * &automaton.epsilon_closure)
                .unwrap(),
        }
    }

    pub fn step(&mut self, c: char) -> Option<()> {
        if self.active.val_indx.first() != Some(&0) {
            self.active.val_indx.insert(0, 0);
        }
        self.active = (&self.active * &self.automaton.epsilon_closure)?;
        self.active = (&self.active * self.automaton.transition.get(&c)?)?;
        self.active = (&self.active * &self.automaton.epsilon_closure)?;
        Some(())
    }

    pub fn is_accepting(&self) -> bool {
        let mut state_indx = 0;
        let mut accept_indx = 0;

        loop {
            let Some(&state) = self.active.val_indx.get(state_indx) else {
                break;
            };
            let Some(&accept) = self.automaton.final_states.val_indx.get(accept_indx) else {
                break;
            };

            if state == accept {
                return true;
            } else if state > accept {
                accept_indx += 1
            } else {
                state_indx += 1
            }
        }

        false
    }

    pub fn recognizes(&mut self, input: &str) -> bool {
        for c in input.chars() {
            if self.step(c).is_none() {
                self.active = BoolVec::from_indices(vec![], self.automaton.states).unwrap();
                continue;
            }
            if self.is_accepting() {
                return true;
            }
        }
        self.is_accepting()
    }
}

#[cfg(test)]
mod match_tests {
    use super::*;

    fn recognizes(automaton: &Automaton, input: &str) -> bool {
        let mut m = Match::new(automaton);
        for c in input.chars() {
            if m.step(c).is_none() {
                return false;
            }
        }
        m.is_accepting()
    }

    // ── Literal ──

    #[test]
    fn literal_accepts_single_char() {
        let nfa = Automaton::literal('a');
        assert!(recognizes(&nfa, "a"));
    }

    #[test]
    fn literal_rejects_wrong_char() {
        let nfa = Automaton::literal('a');
        assert!(!recognizes(&nfa, "b"));
    }

    #[test]
    fn literal_rejects_empty() {
        let nfa = Automaton::literal('a');
        assert!(!recognizes(&nfa, ""));
    }

    #[test]
    fn literal_rejects_too_long() {
        let nfa = Automaton::literal('a');
        assert!(!recognizes(&nfa, "aa"));
    }

    // ── Empty ──

    #[test]
    fn empty_accepts_empty_string() {
        let nfa = Automaton::empty();
        assert!(recognizes(&nfa, ""));
    }

    // ── Concat ──

    #[test]
    fn concat_accepts_sequence() {
        let nfa = Automaton::concat(Automaton::literal('a'), Automaton::literal('b'));
        assert!(recognizes(&nfa, "ab"));
    }

    #[test]
    fn concat_rejects_prefix() {
        let nfa = Automaton::concat(Automaton::literal('a'), Automaton::literal('b'));
        assert!(!recognizes(&nfa, "a"));
    }

    #[test]
    fn concat_rejects_reversed() {
        let nfa = Automaton::concat(Automaton::literal('a'), Automaton::literal('b'));
        assert!(!recognizes(&nfa, "ba"));
    }

    #[test]
    fn concat_rejects_empty() {
        let nfa = Automaton::concat(Automaton::literal('a'), Automaton::literal('b'));
        assert!(!recognizes(&nfa, ""));
    }

    // ── Union ──

    #[test]
    fn union_accepts_either_branch() {
        let nfa = Automaton::union(Automaton::literal('a'), Automaton::literal('b'));
        assert!(recognizes(&nfa, "a"));
        assert!(recognizes(&nfa, "b"));
    }

    #[test]
    fn union_rejects_concat_of_branches() {
        let nfa = Automaton::union(Automaton::literal('a'), Automaton::literal('b'));
        assert!(!recognizes(&nfa, "ab"));
    }

    #[test]
    fn union_rejects_empty() {
        let nfa = Automaton::union(Automaton::literal('a'), Automaton::literal('b'));
        assert!(!recognizes(&nfa, ""));
    }

    // ── Kleene star ──

    #[test]
    fn star_accepts_empty() {
        let nfa = Automaton::literal('a').kleene_star();
        assert!(recognizes(&nfa, ""));
    }

    #[test]
    fn star_accepts_one() {
        let nfa = Automaton::literal('a').kleene_star();
        assert!(recognizes(&nfa, "a"));
    }

    #[test]
    fn star_accepts_many() {
        let nfa = Automaton::literal('a').kleene_star();
        assert!(recognizes(&nfa, "aaaaaaa"));
    }

    #[test]
    fn star_rejects_wrong_char() {
        let nfa = Automaton::literal('a').kleene_star();
        assert!(!recognizes(&nfa, "aab"));
    }

    // ── Complex: (a|b)*abb ──

    #[test]
    fn textbook_nfa_accepts_abb() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(recognizes(&nfa, "abb"));
    }

    #[test]
    fn textbook_nfa_accepts_aabb() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(recognizes(&nfa, "aabb"));
    }

    #[test]
    fn textbook_nfa_accepts_babb() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(recognizes(&nfa, "babb"));
    }

    #[test]
    fn textbook_nfa_rejects_ab() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(!recognizes(&nfa, "ab"));
    }

    #[test]
    fn textbook_nfa_rejects_empty() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(!recognizes(&nfa, ""));
    }

    #[test]
    fn textbook_nfa_rejects_abba() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a|b)*abb").unwrap());
        assert!(!recognizes(&nfa, "abba"));
    }

    // ── Nested star: (a*b*)* ──

    #[test]
    fn nested_star_accepts_mixed() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a*b*)*").unwrap());
        assert!(recognizes(&nfa, ""));
        assert!(recognizes(&nfa, "a")); // fails
        assert!(recognizes(&nfa, "b"));
        assert!(recognizes(&nfa, "aaabbb"));
        assert!(recognizes(&nfa, "abab"));
        assert!(recognizes(&nfa, "bba"));
    }

    #[test]
    fn nested_star_rejects_foreign_char() {
        let nfa = Automaton::from_regex_node(RegexNode::parse("(a*b*)*").unwrap());
        assert!(!recognizes(&nfa, "abc"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn recognizes(re: &str, input: &str) -> bool {
        let node = RegexNode::parse(re).expect(&format!("Failed to parse regex: {re}"));
        let nfa = Automaton::from_regex_node(node);
        let mut m = Match::new(&nfa);
        for c in input.chars() {
            if m.step(c).is_none() {
                return false;
            }
        }
        m.is_accepting()
    }

    // ── Union of concats ──

    #[test]
    fn union_of_concats_accepts_both() {
        assert!(recognizes("ab|cd", "ab"));
        assert!(recognizes("ab|cd", "cd"));
    }

    #[test]
    fn union_of_concats_rejects_cross() {
        assert!(!recognizes("ab|cd", "ac"));
        assert!(!recognizes("ab|cd", "bd"));
        assert!(!recognizes("ab|cd", "abcd"));
    }

    // ── Star of concat ──

    #[test]
    fn star_of_concat_accepts() {
        assert!(recognizes("(ab)*", ""));
        assert!(recognizes("(ab)*", "ab"));
        assert!(recognizes("(ab)*", "abababab"));
    }

    #[test]
    fn star_of_concat_rejects_partial() {
        assert!(!recognizes("(ab)*", "a"));
        assert!(!recognizes("(ab)*", "aba"));
        assert!(!recognizes("(ab)*", "abba"));
    }

    // ── Deeply nested ──

    #[test]
    fn nested_star_union_concat() {
        assert!(recognizes("((a|b)*c)*", ""));
        assert!(recognizes("((a|b)*c)*", "c"));
        assert!(recognizes("((a|b)*c)*", "ac"));
        assert!(recognizes("((a|b)*c)*", "abcbc"));
        assert!(recognizes("((a|b)*c)*", "aabbcc"));
    }

    #[test]
    fn nested_star_union_concat_rejects() {
        assert!(!recognizes("((a|b)*c)*", "a"));
        assert!(!recognizes("((a|b)*c)*", "abcb"));
        assert!(!recognizes("((a|b)*c)*", "abcca"));
    }

    // ── Empty in combinations ──

    #[test]
    fn empty_union_left() {
        // "(|a)b" means (ε|a) followed by b
        assert!(recognizes("(|a)b", "b"));
        assert!(recognizes("(|a)b", "ab"));
        assert!(!recognizes("(|a)b", ""));
        assert!(!recognizes("(|a)b", "aab"));
    }

    #[test]
    fn empty_union_right() {
        assert!(recognizes("(a|)b", "b"));
        assert!(recognizes("(a|)b", "ab"));
        assert!(!recognizes("(a|)b", ""));
    }

    #[test]
    fn empty_star() {
        // Star of empty should accept only empty string
        let node = RegexNode::KleeneStar(Box::new(RegexNode::Empty));
        let nfa = Automaton::from_regex_node(node);
        let m = Match::new(&nfa);
        assert!(m.is_accepting());
    }

    // ── Three-way union ──

    #[test]
    fn three_way_union() {
        assert!(recognizes("a|b|c", "a"));
        assert!(recognizes("a|b|c", "b"));
        assert!(recognizes("a|b|c", "c"));
        assert!(!recognizes("a|b|c", "d"));
        assert!(!recognizes("a|b|c", "ab"));
    }

    // ── Long concat chains ──

    #[test]
    fn four_char_concat() {
        assert!(recognizes("abcd", "abcd"));
        assert!(!recognizes("abcd", "abc"));
        assert!(!recognizes("abcd", "abcde"));
        assert!(!recognizes("abcd", "abdc"));
    }

    #[test]
    fn six_char_concat() {
        assert!(recognizes("abcdef", "abcdef"));
        assert!(!recognizes("abcdef", "abcde"));
    }

    // ── Star interacting with concat ──

    #[test]
    fn star_then_literal() {
        assert!(recognizes("a*b", "b"));
        assert!(recognizes("a*b", "ab"));
        assert!(recognizes("a*b", "aaab"));
        assert!(!recognizes("a*b", ""));
        assert!(!recognizes("a*b", "a"));
        assert!(!recognizes("a*b", "ba"));
    }

    #[test]
    fn literal_then_star() {
        assert!(recognizes("ab*", "a"));
        assert!(recognizes("ab*", "ab"));
        assert!(recognizes("ab*", "abbb"));
        assert!(!recognizes("ab*", ""));
        assert!(!recognizes("ab*", "b"));
    }

    #[test]
    fn star_between_literals() {
        assert!(recognizes("ab*c", "ac"));
        assert!(recognizes("ab*c", "abc"));
        assert!(recognizes("ab*c", "abbbc"));
        assert!(!recognizes("ab*c", "abbb"));
        assert!(!recognizes("ab*c", "c"));
    }

    // ── Multiple stars ──

    #[test]
    fn two_consecutive_stars() {
        assert!(recognizes("a*b*", ""));
        assert!(recognizes("a*b*", "aaa"));
        assert!(recognizes("a*b*", "bbb"));
        assert!(recognizes("a*b*", "aaabbb"));
        assert!(!recognizes("a*b*", "ba"));
        assert!(!recognizes("a*b*", "aba"));
    }

    // ── Union with star branches ──

    #[test]
    fn union_of_stars() {
        assert!(recognizes("a*|b*", ""));
        assert!(recognizes("a*|b*", "aaa"));
        assert!(recognizes("a*|b*", "bbb"));
        // "ab" is NOT in a*|b* since union picks one branch
        assert!(!recognizes("a*|b*", "ab"));
    }

    // ── Larger alphabet ──

    #[test]
    fn digits_in_regex() {
        assert!(recognizes("(0|1)*", ""));
        assert!(recognizes("(0|1)*", "0110100"));
        assert!(!recognizes("(0|1)*", "012"));
    }

    #[test]
    fn mixed_alpha_digit() {
        assert!(recognizes("a1b2", "a1b2"));
        assert!(!recognizes("a1b2", "a1b"));
        assert!(!recognizes("a1b2", "ab12"));
    }

    // ── Star of union of concats ──

    #[test]
    fn star_of_union_of_concats() {
        // (ab|cd)* — alternating pairs
        assert!(recognizes("(ab|cd)*", ""));
        assert!(recognizes("(ab|cd)*", "ab"));
        assert!(recognizes("(ab|cd)*", "cd"));
        assert!(recognizes("(ab|cd)*", "abcd"));
        assert!(recognizes("(ab|cd)*", "cdab"));
        assert!(recognizes("(ab|cd)*", "ababcdcd"));
        assert!(!recognizes("(ab|cd)*", "ac"));
        assert!(!recognizes("(ab|cd)*", "a"));
    }

    // ── Triple nesting ──

    #[test]
    fn triple_nested_star() {
        // (((a)*))* is just a*
        assert!(recognizes("(((a)*))*", ""));
        assert!(recognizes("(((a)*))*", "a"));
        assert!(recognizes("(((a)*))*", "aaaa"));
        assert!(!recognizes("(((a)*))*", "b"));
    }

    // ── Same automaton, multiple matches ──

    #[test]
    fn reuse_automaton() {
        let node = RegexNode::parse("(a|b)*abb").unwrap();
        let nfa = Automaton::from_regex_node(node);

        let inputs = vec![
            ("abb", true),
            ("aabb", true),
            ("babb", true),
            ("bbbabb", true),
            ("ab", false),
            ("", false),
            ("abba", false),
            ("abb abb", false),
        ];

        for (input, expected) in inputs {
            let mut m = Match::new(&nfa);
            let mut ok = true;
            for c in input.chars() {
                if m.step(c).is_none() {
                    ok = false;
                    break;
                }
            }
            let result = ok && m.is_accepting();
            assert_eq!(result, expected, "Failed on input: {input:?}");
        }
    }
}
