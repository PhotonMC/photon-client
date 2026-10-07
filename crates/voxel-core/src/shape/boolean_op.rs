/// Errors raised while constructing a shape with reversed bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeError {
    ReversedBounds,
}

/// Boolean operation supported by the current shape join slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    And,
    False,
    First,
    NotSame,
    OnlyFirst,
    OnlySecond,
    Or,
    Second,
}

impl BooleanOp {
    pub(super) fn apply(self, first: bool, second: bool) -> bool {
        match self {
            Self::And => first && second,
            Self::False => false,
            Self::First => first,
            Self::NotSame => first != second,
            Self::OnlyFirst => first && !second,
            Self::OnlySecond => second && !first,
            Self::Or => first || second,
            Self::Second => second,
        }
    }
}
