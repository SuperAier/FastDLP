/// Simplified regex tree implementation
/// This module provides basic regex tree structures for more advanced regex generation
/// For this initial implementation, we keep it simple and focus on string-based patterns

use crate::common::*;
use crate::error::Result;

/// Represents a node in the regex tree
#[derive(Debug, Clone)]
pub enum RegexNode {
    /// Literal character or string
    Literal(String),
    /// Character class [abc]
    CharClass(Vec<char>),
    /// Concatenation of nodes
    Concat(Vec<RegexNode>),
    /// Alternation (OR)
    Alt(Vec<RegexNode>),
    /// Quantifier (* + ? {n,m})
    Quantifier(Box<RegexNode>, QuantifierType),
    /// Group (capturing or non-capturing)
    Group(Box<RegexNode>, bool), // bool indicates if capturing
    /// Anchor (^ $)
    Anchor(AnchorType),
    /// Dot (any character)
    Dot,
}

/// Types of quantifiers
#[derive(Debug, Clone)]
pub enum QuantifierType {
    ZeroOrMore,    // *
    OneOrMore,     // +
    ZeroOrOne,     // ?
    Exact(usize),  // {n}
    Range(usize, Option<usize>), // {n,m} or {n,}
}

/// Types of anchors
#[derive(Debug, Clone)]
pub enum AnchorType {
    Start,  // ^
    End,    // $
    WordBoundary, // \b
}

impl RegexNode {
    /// Convert the regex tree to a string representation
    pub fn to_string(&self) -> String {
        match self {
            RegexNode::Literal(s) => regex::escape(s),
            RegexNode::CharClass(chars) => {
                let char_str: String = chars.iter().collect();
                format!("[{}]", char_str)
            }
            RegexNode::Concat(nodes) => {
                nodes.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("")
            }
            RegexNode::Alt(nodes) => {
                nodes.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("|")
            }
            RegexNode::Quantifier(node, quant) => {
                let node_str = node.to_string();
                let quant_str = match quant {
                    QuantifierType::ZeroOrMore => "*",
                    QuantifierType::OneOrMore => "+",
                    QuantifierType::ZeroOrOne => "?",
                    QuantifierType::Exact(n) => return format!("({}){{{}}}", node_str, n),
                    QuantifierType::Range(n, Some(m)) => return format!("({}){{{},{}}}", node_str, n, m),
                    QuantifierType::Range(n, None) => return format!("({}){{{},}}", node_str, n),
                };
                format!("({}){}", node_str, quant_str)
            }
            RegexNode::Group(node, capturing) => {
                let node_str = node.to_string();
                if *capturing {
                    format!("({})", node_str)
                } else {
                    format!("(?:{})", node_str)
                }
            }
            RegexNode::Anchor(anchor) => match anchor {
                AnchorType::Start => "^".to_string(),
                AnchorType::End => "$".to_string(),
                AnchorType::WordBoundary => r"\b".to_string(),
            },
            RegexNode::Dot => ".".to_string(),
        }
    }

    /// Get the depth of the regex tree
    pub fn depth(&self) -> usize {
        match self {
            RegexNode::Literal(_) | RegexNode::CharClass(_) | RegexNode::Anchor(_) | RegexNode::Dot => 1,
            RegexNode::Concat(nodes) | RegexNode::Alt(nodes) => {
                1 + nodes.iter().map(|n| n.depth()).max().unwrap_or(0)
            }
            RegexNode::Quantifier(node, _) | RegexNode::Group(node, _) => 1 + node.depth(),
        }
    }

    /// Check if the tree is valid (no cycles, reasonable depth)
    pub fn is_valid(&self) -> bool {
        self.depth() <= MAX_REGEX_TREE_DEPTH
    }
}

/// Builder for creating regex trees
pub struct RegexTreeBuilder {
    root: Option<RegexNode>,
}

impl RegexTreeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Add a literal string
    pub fn literal(mut self, s: &str) -> Self {
        let node = RegexNode::Literal(s.to_string());
        self.root = Some(self.combine_with_root(node));
        self
    }

    /// Add a character class
    pub fn char_class(mut self, chars: Vec<char>) -> Self {
        let node = RegexNode::CharClass(chars);
        self.root = Some(self.combine_with_root(node));
        self
    }

    /// Add a quantifier to the last node
    pub fn quantifier(mut self, quant: QuantifierType) -> Self {
        if let Some(root) = self.root.take() {
            let node = RegexNode::Quantifier(Box::new(root), quant);
            self.root = Some(node);
        }
        self
    }

    /// Add an anchor
    pub fn anchor(mut self, anchor: AnchorType) -> Self {
        let node = RegexNode::Anchor(anchor);
        self.root = Some(self.combine_with_root(node));
        self
    }

    /// Build the final regex tree
    pub fn build(self) -> Result<RegexNode> {
        self.root.ok_or_else(|| crate::error::FastDlpError::analysis_error("Empty regex tree"))
    }

    /// Combine new node with existing root
    fn combine_with_root(&self, new_node: RegexNode) -> RegexNode {
        match &self.root {
            Some(existing) => RegexNode::Concat(vec![existing.clone(), new_node]),
            None => new_node,
        }
    }
}

impl Default for RegexTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_node() {
        let node = RegexNode::Literal("hello".to_string());
        assert_eq!(node.to_string(), "hello");
        assert_eq!(node.depth(), 1);
        assert!(node.is_valid());
    }

    #[test]
    fn test_char_class_node() {
        let node = RegexNode::CharClass(vec!['a', 'b', 'c']);
        assert_eq!(node.to_string(), "[abc]");
        assert_eq!(node.depth(), 1);
    }

    #[test]
    fn test_quantifier_node() {
        let inner = RegexNode::Literal("a".to_string());
        let node = RegexNode::Quantifier(Box::new(inner), QuantifierType::OneOrMore);
        assert_eq!(node.to_string(), "(a)+");
        assert_eq!(node.depth(), 2);
    }

    #[test]
    fn test_concat_node() {
        let node1 = RegexNode::Literal("hello".to_string());
        let node2 = RegexNode::Literal("world".to_string());
        let concat = RegexNode::Concat(vec![node1, node2]);
        assert_eq!(concat.to_string(), "helloworld");
        assert_eq!(concat.depth(), 2);
    }

    #[test]
    fn test_alt_node() {
        let node1 = RegexNode::Literal("hello".to_string());
        let node2 = RegexNode::Literal("world".to_string());
        let alt = RegexNode::Alt(vec![node1, node2]);
        assert_eq!(alt.to_string(), "hello|world");
        assert_eq!(alt.depth(), 2);
    }

    #[test]
    fn test_builder() {
        let tree = RegexTreeBuilder::new()
            .anchor(AnchorType::Start)
            .literal("hello")
            .quantifier(QuantifierType::OneOrMore)
            .anchor(AnchorType::End)
            .build()
            .unwrap();

        let pattern = tree.to_string();
        assert!(pattern.contains("hello"));
        assert!(pattern.contains("^"));
        assert!(pattern.contains("$"));
    }

    #[test]
    fn test_depth_limit() {
        let mut node = RegexNode::Literal("a".to_string());
        
        // Create a deeply nested structure
        for _ in 0..MAX_REGEX_TREE_DEPTH + 1 {
            node = RegexNode::Group(Box::new(node), false);
        }
        
        assert!(!node.is_valid());
    }
} 