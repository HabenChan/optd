use crate::nodes::NodeType;

pub enum RuleMatcher<T: NodeType> {
    /// Match a node of type `typ`.
    MatchNode { typ: T, children: Vec<Self> },
    /// Match "discriminant" (Only check for variant matches---don't consider
    /// inner data).
    /// This may be useful when, for example, one has an enum variant such as
    /// ConstantExpr(ConstantType), and one wants to match on all ConstantExpr
    /// regardless of the inner ConstantType.
    MatchDiscriminant {
        typ_discriminant: std::mem::Discriminant<T>,
        children: Vec<Self>,
    },
    /// Match any plan node.
    Any,
    /// Match all plan node.
    AnyMany,
}
