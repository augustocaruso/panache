//! Citation AST node wrappers.

use super::{AstNode, PanacheLanguage, SyntaxKind, SyntaxNode, SyntaxToken};

pub struct Citation(SyntaxNode);

impl AstNode for Citation {
    type Language = PanacheLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CITATION
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self(syntax))
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl Citation {
    /// Whether the citation has its own enclosing square brackets.
    pub fn is_bracketed(&self) -> bool {
        self.0
            .first_token()
            .is_some_and(|token| token.kind() == SyntaxKind::LINK_START)
    }

    /// Every key marker, including markers whose braced key is empty.
    pub fn markers(&self) -> impl Iterator<Item = CitationMarker> {
        self.0
            .children_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == SyntaxKind::CITATION_MARKER)
            .map(CitationMarker)
    }

    pub fn keys(&self) -> Vec<CitationKey> {
        self.0
            .children_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == SyntaxKind::CITATION_KEY)
            .map(CitationKey)
            .collect()
    }

    pub fn key_texts(&self) -> Vec<String> {
        self.keys().into_iter().map(|key| key.text()).collect()
    }
}

pub struct CitationMarker(SyntaxToken);

impl CitationMarker {
    /// Range of just the `@` or `-@` marker.
    pub fn marker_range(&self) -> rowan::TextRange {
        self.0.text_range()
    }

    pub fn key(&self) -> Option<CitationKey> {
        let mut next = self.0.next_sibling_or_token()?.into_token()?;
        if next.kind() == SyntaxKind::CITATION_BRACE_OPEN {
            next = next.next_sibling_or_token()?.into_token()?;
        }
        (next.kind() == SyntaxKind::CITATION_KEY).then_some(CitationKey(next))
    }

    /// Range of the marker and its key, including any enclosing braces.
    pub fn text_range(&self) -> rowan::TextRange {
        if let Some(key) = self.key() {
            return key.marked_range();
        }
        let mut end = self.0.text_range().end();
        let next = self
            .0
            .next_sibling_or_token()
            .and_then(|it| it.into_token());
        if let Some(open) = next
            && open.kind() == SyntaxKind::CITATION_BRACE_OPEN
        {
            end = open.text_range().end();
            if let Some(close) = open.next_sibling_or_token().and_then(|it| it.into_token())
                && close.kind() == SyntaxKind::CITATION_BRACE_CLOSE
            {
                end = close.text_range().end();
            }
        }
        rowan::TextRange::new(self.0.text_range().start(), end)
    }
}

pub struct CitationKey(SyntaxToken);

impl CitationKey {
    pub fn text(&self) -> String {
        self.0.text().to_string()
    }

    pub fn text_range(&self) -> rowan::TextRange {
        self.0.text_range()
    }

    /// Range covering this key *with* its `@`/`-@` marker and, for the braced
    /// form, the surrounding braces --- `@key`, `-@key`, `@{key with spaces}`.
    ///
    /// [`text_range`](Self::text_range) covers the bare key text, which is what
    /// a rename edit must replace. This is the span to underline in a
    /// diagnostic: it points at one key inside a multi-key citation such as
    /// `[@a; @b]` instead of the whole bracketed group.
    pub fn marked_range(&self) -> rowan::TextRange {
        let range = self.0.text_range();
        let mut start = range.start();
        let mut end = range.end();

        let mut prev = self
            .0
            .prev_sibling_or_token()
            .and_then(|it| it.into_token());
        if let Some(token) = &prev
            && token.kind() == SyntaxKind::CITATION_BRACE_OPEN
        {
            start = token.text_range().start();
            prev = token.prev_sibling_or_token().and_then(|it| it.into_token());
        }
        if let Some(token) = prev
            && token.kind() == SyntaxKind::CITATION_MARKER
        {
            start = token.text_range().start();
        }

        if let Some(token) = self
            .0
            .next_sibling_or_token()
            .and_then(|it| it.into_token())
            && token.kind() == SyntaxKind::CITATION_BRACE_CLOSE
        {
            end = token.text_range().end();
        }

        rowan::TextRange::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn marked_spans(input: &str) -> Vec<&str> {
        parse(input, None)
            .descendants()
            .filter_map(Citation::cast)
            .flat_map(|citation| citation.keys())
            .map(|key| {
                let range = key.marked_range();
                &input[usize::from(range.start())..usize::from(range.end())]
            })
            .collect()
    }

    #[test]
    fn marked_range_covers_one_key_of_a_group() {
        assert_eq!(marked_spans("Text [@a; @b].\n"), vec!["@a", "@b"]);
    }

    #[test]
    fn marked_range_covers_marker_variants() {
        assert_eq!(marked_spans("A [-@foo] b @bar c.\n"), vec!["-@foo", "@bar"]);
    }

    #[test]
    fn marked_range_covers_braced_keys() {
        assert_eq!(marked_spans("A [@{weird key}].\n"), vec!["@{weird key}"]);
    }

    #[test]
    fn markers_include_empty_keys() {
        let input = "[@a; -@{}] @{}\n";
        let citations: Vec<_> = parse(input, None)
            .descendants()
            .filter_map(Citation::cast)
            .collect();
        assert!(citations[0].is_bracketed());
        assert!(!citations[1].is_bracketed());
        let markers: Vec<_> = citations.iter().flat_map(Citation::markers).collect();
        let spans: Vec<_> = markers
            .iter()
            .map(|marker| {
                let range = marker.text_range();
                &input[usize::from(range.start())..usize::from(range.end())]
            })
            .collect();
        assert_eq!(spans, ["@a", "-@{}", "@{}"]);
        assert_eq!(markers[0].key().unwrap().text(), "a");
        assert!(markers[1].key().is_none());
        assert_eq!(markers[2].key().unwrap().text(), "");
    }
}
