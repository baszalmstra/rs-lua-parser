mod generated;

use crate::{syntax_node::SyntaxNodeChildren, SmolStr, SyntaxNode, SyntaxToken, TreeArc};

pub use self::{generated::*};

use std::marker::PhantomData;

/// The main trait to go from untyped `SyntaxNode` to a typed ast. The conversion itself has zero
/// runtime cost; ast and syntax nodes have exactly the same representation; a pointer to the tree
/// root and a pointer to the node itself.
pub trait AstNode:
rowan::TransparentNewType<Repr = rowan::SyntaxNode> + ToOwned<Owned = TreeArc<Self>>
{
    fn cast(syntax: &SyntaxNode) -> Option<&Self>
        where
            Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}

/// Like an `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken<'a> {
    fn cast(token: SyntaxToken<'a>) -> Option<Self>
        where
            Self: Sized;
    fn syntax(&self) -> SyntaxToken<'a>;
    fn text(&self) -> &'a SmolStr {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug)]
pub struct AstChildren<'a, N> {
    inner: SyntaxNodeChildren<'a>,
    ph: PhantomData<N>,
}

impl<'a, N> AstChildren<'a, N> {
    fn new(parent: &'a SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<'a, N: AstNode + 'a> Iterator for AstChildren<'a, N> {
    type Item = &'a N;
    fn next(&mut self) -> Option<&'a N> {
        self.inner.by_ref().find_map(N::cast)
    }
}

fn child_opt<P: AstNode, C: AstNode>(parent: &P) -> Option<&C> {
    children(parent).next()
}

fn children<P: AstNode, C: AstNode>(parent: &P) -> AstChildren<C> {
    AstChildren::new(parent.syntax())
}
