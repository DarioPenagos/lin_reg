use std::mem;

#[derive(Debug, Clone)]
pub enum RegexNode {
    Literal(char),
    Concat(Box<Self>, Box<Self>),
    Union(Box<Self>, Box<Self>),
    KleeneStar(Box<Self>),
    Empty,
}

impl RegexNode {
    pub fn parse(re: &str) -> Option<RegexNode> {
        let mut concat_nodes = Vec::new();
        let mut char_itr = re.chars();

        while let Some(c) = char_itr.next() {
            if c.is_alphabetic() || c.is_ascii_digit() {
                concat_nodes.push(RegexNode::Literal(c))
            } else if c == '|' {
                let next_node = Self::parse(char_itr.as_str())?;
                let last_node = concat_list(mem::replace(&mut concat_nodes, Vec::new()));
                return Some(RegexNode::Union(Box::new(last_node), Box::new(next_node)));
            } else if c == '*' {
                let last_node = concat_nodes.pop()?;
                concat_nodes.push(RegexNode::KleeneStar(Box::new(last_node)));
            } else if c == '(' {
                let mut sub_str = String::new();
                let mut depth = 0;
                loop {
                    match char_itr.next() {
                        Some(c) => {
                            if c == ')' && depth == 0 {
                                concat_nodes.push(Self::parse(sub_str.as_str())?);
                                break;
                            } else if c == ')' {
                                sub_str.push(c);
                                depth -= 1;
                            } else if c == '(' {
                                sub_str.push(c);
                                depth += 1;
                            } else {
                                sub_str.push(c);
                            }
                        }
                        None => return None,
                    }
                }
            } else {
                return None;
            };
        }

        Some(concat_list(concat_nodes))
    }
}

fn concat_list(list: Vec<RegexNode>) -> RegexNode {
    list.into_iter()
        .rev()
        .reduce(|right, left| RegexNode::Concat(Box::new(left), Box::new(right)))
        .unwrap_or(RegexNode::Empty)
}
