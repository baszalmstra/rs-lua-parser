use crate::{ParseError, TextRange, TextUnit};
use std::cmp::min;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    Offset(TextUnit),
    Range(TextRange),
}

impl Into<Location> for TextUnit {
    fn into(self) -> Location {
        Location::Offset(self)
    }
}

impl Into<Location> for TextRange {
    fn into(self) -> Location {
        Location::Range(self)
    }
}

impl Location {
    pub fn offset(&self) -> TextUnit {
        match &self {
            Location::Offset(offset) => *offset,
            Location::Range(range) => range.start(),
        }
    }

    pub fn add_offset(&self, plus_offset: TextUnit, minus_offset: TextUnit) -> Location {
        match &self {
            Location::Range(range) => Location::Range(range + plus_offset - minus_offset),
            Location::Offset(offset) => Location::Offset(offset + plus_offset - minus_offset),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxError {
    kind: SyntaxErrorKind,
    location: Location,
}

impl SyntaxError {
    pub fn new<L: Into<Location>>(kind: SyntaxErrorKind, loc: L) -> SyntaxError {
        SyntaxError {
            kind,
            location: loc.into(),
        }
    }

    pub fn kind(&self) -> SyntaxErrorKind {
        self.kind.clone()
    }

    pub fn location(&self) -> Location {
        self.location.clone()
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyntaxErrorKind {
    ParseError(ParseError),
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SyntaxErrorKind::*;
        match self {
            ParseError(msg) => write!(f, "{}", msg.0),
        }
    }
}