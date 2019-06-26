//! This module defines Concrete Syntax Tree (CST), used by Mun.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! modules just wraps its API.

use std::{
    any::Any,
    borrow::Borrow,
    fmt::{self, Write},
    iter::successors,
};

use crate::{
    ParseError,
    syntax_error::{SyntaxError, SyntaxErrorKind},
    AstNode, SmolStr, Chunk, SyntaxKind, SyntaxText, TextRange, TextUnit,
};
use rowan::{GreenNodeBuilder, TransparentNewType};

pub use rowan::WalkEvent;
pub(crate) use rowan::{GreenNode, GreenToken};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InsertPosition<T> {
    First,
    Last,
    Before(T),
    After(T),
}

/// Marker trait for CST and AST nodes
pub trait SyntaxNodeWrapper: TransparentNewType<Repr = rowan::SyntaxNode> {}
impl<T: TransparentNewType<Repr = rowan::SyntaxNode>> SyntaxNodeWrapper for T {}

/// An owning smart pointer for CST or AST nodes.
#[derive(PartialEq, Eq, Hash)]
pub struct TreeArc<T: SyntaxNodeWrapper>(pub(crate) rowan::TreeArc<T>);

impl<T: SyntaxNodeWrapper> Borrow<T> for TreeArc<T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<T> TreeArc<T>
    where
        T: SyntaxNodeWrapper,
{
    pub(crate) fn cast<U>(this: TreeArc<T>) -> TreeArc<U>
        where
            U: SyntaxNodeWrapper,
    {
        TreeArc(rowan::TreeArc::cast(this.0))
    }
}

impl<T> std::ops::Deref for TreeArc<T>
    where
        T: SyntaxNodeWrapper,
{
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T> PartialEq<T> for TreeArc<T>
    where
        T: SyntaxNodeWrapper,
        T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        let t: &T = self;
        t == other
    }
}

impl<T> Clone for TreeArc<T>
    where
        T: SyntaxNodeWrapper,
{
    fn clone(&self) -> TreeArc<T> {
        TreeArc(self.0.clone())
    }
}

impl<T> fmt::Debug for TreeArc<T>
    where
        T: SyntaxNodeWrapper,
        T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SyntaxNode(pub(crate) rowan::SyntaxNode);
unsafe impl TransparentNewType for SyntaxNode {
    type Repr = rowan::SyntaxNode;
}

impl ToOwned for SyntaxNode {
    type Owned = TreeArc<SyntaxNode>;
    fn to_owned(&self) -> TreeArc<SyntaxNode> {
        let ptr = TreeArc(self.0.to_owned());
        TreeArc::cast(ptr)
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}@{:?}", self.kind(), self.range())
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.text(), fmt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Prev,
}

impl SyntaxNode {
    pub(crate) fn new(green: GreenNode, errors: Vec<SyntaxError>) -> TreeArc<SyntaxNode> {
        let errors: Option<Box<Any + Send + Sync>> = if errors.is_empty() {
            None
        } else {
            Some(Box::new(errors))
        };
        let ptr = TreeArc(rowan::SyntaxNode::new(green, errors));
        TreeArc::cast(ptr)
    }

    pub fn kind(&self) -> SyntaxKind {
        self.0.kind().0.into()
    }

    pub fn range(&self) -> TextRange {
        self.0.range()
    }

    pub fn text(&self) -> SyntaxText {
        SyntaxText::new(self)
    }

    pub fn parent(&self) -> Option<&SyntaxNode> {
        self.0.parent().map(SyntaxNode::from_repr)
    }

    pub fn first_child(&self) -> Option<&SyntaxNode> {
        self.0.first_child().map(SyntaxNode::from_repr)
    }

    pub fn first_child_or_token(&self) -> Option<SyntaxElement> {
        self.0.first_child_or_token().map(SyntaxElement::from)
    }

    pub fn last_child(&self) -> Option<&SyntaxNode> {
        self.0.last_child().map(SyntaxNode::from_repr)
    }

    pub fn last_child_or_token(&self) -> Option<SyntaxElement> {
        self.0.last_child_or_token().map(SyntaxElement::from)
    }

    pub fn next_sibling(&self) -> Option<&SyntaxNode> {
        self.0.next_sibling().map(SyntaxNode::from_repr)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.0.next_sibling_or_token().map(SyntaxElement::from)
    }

    pub fn prev_sibling(&self) -> Option<&SyntaxNode> {
        self.0.prev_sibling().map(SyntaxNode::from_repr)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.0.prev_sibling_or_token().map(SyntaxElement::from)
    }

    pub fn children(&self) -> SyntaxNodeChildren {
        SyntaxNodeChildren(self.0.children())
    }

    pub fn children_with_tokens(&self) -> SyntaxElementChildren {
        SyntaxElementChildren(self.0.children_with_tokens())
    }

    pub fn first_token(&self) -> Option<SyntaxToken> {
        self.0.first_token().map(SyntaxToken::from)
    }

    pub fn last_token(&self) -> Option<SyntaxToken> {
        self.0.last_token().map(SyntaxToken::from)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = &SyntaxNode> {
        successors(Some(self), |&node| node.parent())
    }

    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> {
        self.preorder_with_tokens().filter_map(|event| match event {
            WalkEvent::Enter(it) => Some(it),
            WalkEvent::Leave(_) => None,
        })
    }

    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = &SyntaxNode> {
        successors(Some(self), move |&node| match direction {
            Direction::Next => node.next_sibling(),
            Direction::Prev => node.prev_sibling(),
        })
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement> {
        let me: SyntaxElement = self.into();
        successors(Some(me), move |el| match direction {
            Direction::Next => el.next_sibling_or_token(),
            Direction::Prev => el.prev_sibling_or_token(),
        })
    }

    pub fn preorder(&self) -> impl Iterator<Item = WalkEvent<&SyntaxNode>> {
        self.0.preorder().map(|event| match event {
            WalkEvent::Enter(n) => WalkEvent::Enter(SyntaxNode::from_repr(n)),
            WalkEvent::Leave(n) => WalkEvent::Leave(SyntaxNode::from_repr(n)),
        })
    }

    pub fn preorder_with_tokens(&self) -> impl Iterator<Item = WalkEvent<SyntaxElement>> {
        self.0.preorder_with_tokens().map(|event| match event {
            WalkEvent::Enter(n) => WalkEvent::Enter(n.into()),
            WalkEvent::Leave(n) => WalkEvent::Leave(n.into()),
        })
    }

    pub fn memory_size_of_subtree(&self) -> usize {
        self.0.memory_size_of_subtree()
    }

    pub fn debug_dump(&self) -> String {
        let mut errors: Vec<_> = match self.ancestors().find_map(Chunk::cast) {
            Some(file) => file.errors(),
            None => self.root_data().to_vec(),
        };
        errors.sort_by_key(|e| e.location().offset());
        let mut err_pos = 0;
        let mut level = 0;
        let mut buf = String::new();
        macro_rules! indent {
            () => {
                for _ in 0..level {
                    buf.push_str("  ");
                }
            };
        }

        for event in self.preorder_with_tokens() {
            match event {
                WalkEvent::Enter(element) => {
                    indent!();
                    match element {
                        SyntaxElement::Node(node) => writeln!(buf, "{:?}", node).unwrap(),
                        SyntaxElement::Token(token) => {
                            writeln!(buf, "{:?}", token).unwrap();
                            let off = token.range().end();
                            while err_pos < errors.len() && errors[err_pos].location().offset() <= off {
                                indent!();
                                writeln!(buf, "err: `{}`", errors[err_pos]).unwrap();
                                err_pos += 1;
                            }
                        }
                    }
                    level += 1;
                }
                WalkEvent::Leave(_) => level -= 1,
            }
        }

        assert_eq!(level, 0);
        for err in errors[err_pos..].iter() {
            writeln!(buf, "err: `{}`", err).unwrap();
        }

        buf
    }

    pub(crate) fn root_data(&self) -> &[SyntaxError] {
        match self.0.root_data() {
            None => &[],
            Some(data) => {
                let data: &Vec<SyntaxError> = std::any::Any::downcast_ref(data).unwrap();
                data.as_slice()
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyntaxToken<'a>(pub(crate) rowan::SyntaxToken<'a>);

//FIXME: always output text
impl<'a> fmt::Debug for SyntaxToken<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}@{:?}", self.kind(), self.range())?;
        if self.text().len() < 25 {
            return write!(fmt, " {:?}", self.text());
        }
        let text = self.text().as_str();
        for idx in 21..25 {
            if text.is_char_boundary(idx) {
                let text = format!("{} ...", &text[..idx]);
                return write!(fmt, " {:?}", text);
            }
        }
        unreachable!()
    }
}

impl<'a> fmt::Display for SyntaxToken<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.text(), fmt)
    }
}

impl<'a> From<rowan::SyntaxToken<'a>> for SyntaxToken<'a> {
    fn from(t: rowan::SyntaxToken<'a>) -> Self {
        SyntaxToken(t)
    }
}

impl<'a> SyntaxToken<'a> {
    pub fn kind(&self) -> SyntaxKind {
        self.0.kind().0.into()
    }

    pub fn text(&self) -> &'a SmolStr {
        self.0.text()
    }

    pub fn range(&self) -> TextRange {
        self.0.range()
    }

    pub fn parent(&self) -> &'a SyntaxNode {
        SyntaxNode::from_repr(self.0.parent())
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<'a>> {
        self.0.next_sibling_or_token().map(SyntaxElement::from)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<'a>> {
        self.0.prev_sibling_or_token().map(SyntaxElement::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement<'a>> {
        let me: SyntaxElement = (*self).into();
        successors(Some(me), move |el| match direction {
            Direction::Next => el.next_sibling_or_token(),
            Direction::Prev => el.prev_sibling_or_token(),
        })
    }

    pub fn next_token(&self) -> Option<SyntaxToken<'a>> {
        self.0.next_token().map(SyntaxToken::from)
    }

    pub fn prev_token(&self) -> Option<SyntaxToken<'a>> {
        self.0.prev_token().map(SyntaxToken::from)
    }

    pub(crate) fn replace_with(&self, new_token: GreenToken) -> GreenNode {
        self.0.replace_with(new_token)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SyntaxElement<'a> {
    Node(&'a SyntaxNode),
    Token(SyntaxToken<'a>),
}

impl<'a> fmt::Display for SyntaxElement<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxElement::Node(it) => fmt::Display::fmt(it, fmt),
            SyntaxElement::Token(it) => fmt::Display::fmt(it, fmt),
        }
    }
}

impl<'a> SyntaxElement<'a> {
    pub fn kind(&self) -> SyntaxKind {
        match self {
            SyntaxElement::Node(it) => it.kind(),
            SyntaxElement::Token(it) => it.kind(),
        }
    }

    pub fn as_node(&self) -> Option<&'a SyntaxNode> {
        match self {
            SyntaxElement::Node(node) => Some(*node),
            SyntaxElement::Token(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<SyntaxToken<'a>> {
        match self {
            SyntaxElement::Node(_) => None,
            SyntaxElement::Token(token) => Some(*token),
        }
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<'a>> {
        match self {
            SyntaxElement::Node(it) => it.next_sibling_or_token(),
            SyntaxElement::Token(it) => it.next_sibling_or_token(),
        }
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<'a>> {
        match self {
            SyntaxElement::Node(it) => it.prev_sibling_or_token(),
            SyntaxElement::Token(it) => it.prev_sibling_or_token(),
        }
    }

    pub fn ancestors(&self) -> impl Iterator<Item = &'a SyntaxNode> {
        match self {
            SyntaxElement::Node(it) => it,
            SyntaxElement::Token(it) => it.parent(),
        }
            .ancestors()
    }

    fn text_len(&self) -> TextUnit {
        match self {
            SyntaxElement::Node(node) => node.0.green().text_len(),
            SyntaxElement::Token(token) => TextUnit::of_str(token.0.text()),
        }
    }
}

impl<'a> From<rowan::SyntaxElement<'a>> for SyntaxElement<'a> {
    fn from(el: rowan::SyntaxElement<'a>) -> Self {
        match el {
            rowan::SyntaxElement::Node(n) => SyntaxElement::Node(SyntaxNode::from_repr(n)),
            rowan::SyntaxElement::Token(t) => SyntaxElement::Token(t.into()),
        }
    }
}

impl<'a> From<&'a SyntaxNode> for SyntaxElement<'a> {
    fn from(node: &'a SyntaxNode) -> SyntaxElement<'a> {
        SyntaxElement::Node(node)
    }
}

impl<'a> From<SyntaxToken<'a>> for SyntaxElement<'a> {
    fn from(token: SyntaxToken<'a>) -> SyntaxElement<'a> {
        SyntaxElement::Token(token)
    }
}

impl<'a> SyntaxElement<'a> {
    pub fn range(&self) -> TextRange {
        match self {
            SyntaxElement::Node(it) => it.range(),
            SyntaxElement::Token(it) => it.range(),
        }
    }
}

#[derive(Debug)]
pub struct SyntaxNodeChildren<'a>(rowan::SyntaxNodeChildren<'a>);

impl<'a> Iterator for SyntaxNodeChildren<'a> {
    type Item = &'a SyntaxNode;

    fn next(&mut self) -> Option<&'a SyntaxNode> {
        self.0.next().map(SyntaxNode::from_repr)
    }
}

#[derive(Debug)]
pub struct SyntaxElementChildren<'a>(rowan::SyntaxElementChildren<'a>);

impl<'a> Iterator for SyntaxElementChildren<'a> {
    type Item = SyntaxElement<'a>;

    fn next(&mut self) -> Option<SyntaxElement<'a>> {
        self.0.next().map(SyntaxElement::from)
    }
}

pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder,
}

impl Default for SyntaxTreeBuilder {
    fn default() -> SyntaxTreeBuilder {
        SyntaxTreeBuilder {
            errors: Vec::new(),
            inner: GreenNodeBuilder::new(),
        }
    }
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> TreeArc<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        let node = SyntaxNode::new(green, errors);
        //        if cfg!(debug_assertions) {
        //            crate::validation::validate_block_structure(&node);
        //        }
        node
    }

    pub fn token(&mut self, kind: SyntaxKind, text: SmolStr) {
        self.inner.token(rowan::SyntaxKind(kind.into()), text)
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.inner.start_node(rowan::SyntaxKind(kind.into()))
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node()
    }

    pub fn error(&mut self, error: ParseError, text_pos: TextUnit) {
        let error = SyntaxError::new(SyntaxErrorKind::ParseError(error), text_pos);
        self.errors.push(error)
    }
}