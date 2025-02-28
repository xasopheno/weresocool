use num_rational::Rational64;
use std::cmp::*;
use std::fmt;
use std::hash::*;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FollowAST {
    Follow(Follow),
    Rule(FollowRule),
    Condition(FollowCondition),
    AndCondition(FollowAndCondition),
    SimpleCondition(FollowSimpleCondition),
    Comparison(FollowComparison),
    Action(FollowAction),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CompiledFollowAST {
    FollowNF(FollowNF),
    ConditionNF(ConditionNF),
    ComparisonNF(ComparisonNF),
    ActionNF(ActionNF),
    TransformFn(FollowTransformFn),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub follows: Vec<Follow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Follow {
    pub rules: Vec<FollowRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowRule {
    Conditional {
        condition: FollowCondition,
        action: FollowAction,
    },
    Default(FollowAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowCondition {
    Or(Box<FollowCondition>, Box<FollowAndCondition>),
    And(FollowAndCondition),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowAndCondition {
    And(Box<FollowAndCondition>, Box<FollowSimpleCondition>),
    Simple(FollowSimpleCondition),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowSimpleCondition {
    Comparison(FollowComparison),
    Nested(Box<FollowCondition>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowComparison {
    GreaterThan(FollowVariable, Rational64),
    LessThan(FollowVariable, Rational64),
    EqualTo(FollowVariable, Rational64),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub enum FollowVariable {
    F,
    G,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FollowAction {
    pub expr_f: FollowExpr,
    pub expr_g: FollowExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowExpr {
    Value(Rational64),
    Variable(FollowVariable),
}

#[derive(Clone)]
pub struct FollowTransformFn(pub Arc<dyn Fn(f32, f32) -> f32 + Send + Sync>);

impl FollowTransformFn {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(f32, f32) -> f32 + 'static + Send + Sync,
    {
        Self(Arc::new(func))
    }
}

impl serde::Serialize for FollowTransformFn {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("TransformFn(<function>)")
    }
}

impl<'a> serde::Deserialize<'a> for FollowTransformFn {
    // TODO: This doesn't work
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Ok(FollowTransformFn::new(|_, _| 1.0))
    }
}

impl Eq for FollowTransformFn {}

impl fmt::Debug for FollowTransformFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransformFn(<function>)")
    }
}

fn sample_points() -> &'static [(f32, f32)] {
    &[(1.0, 2.0), (0.0, 0.0), (3.0, -1.0)]
}

impl PartialEq for FollowTransformFn {
    fn eq(&self, other: &Self) -> bool {
        sample_points()
            .iter()
            .all(|&(x, y)| (self.0)(x, y) == (other.0)(x, y))
    }
}

// Implement Hash by hashing the outputs at the same sample points.
// We need some sort of best-effort approach here.
impl std::hash::Hash for FollowTransformFn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for &(x, y) in sample_points() {
            // Convert the output float to bits to have a stable representation.
            // This is naive but at least consistent.
            let output_bits = (self.0)(x, y).to_bits();
            output_bits.hash(state);
        }
    }
}

impl PartialOrd for FollowTransformFn {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for &(x, y) in sample_points() {
            let lhs = (self.0)(x, y);
            let rhs = (other.0)(x, y);

            match lhs.partial_cmp(&rhs) {
                Some(std::cmp::Ordering::Equal) => continue,
                non_eq => return non_eq,
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for FollowTransformFn {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub enum FollowNF {
    Condition {
        test: ConditionNF,
        true_branch: Box<FollowNF>,
        false_branch: Box<FollowNF>,
    },
    Leaf(ActionNF),
}

/// Normal form for conditions
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub enum ConditionNF {
    Comparison(ComparisonNF),
    LogicalAnd(Box<ConditionNF>, Box<ConditionNF>),
    LogicalOr(Box<ConditionNF>, Box<ConditionNF>),
    LogicalNot(Box<ConditionNF>),
}

/// Comparison variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonNF {
    GreaterThan(FollowVariable, f32),
    LessThan(FollowVariable, f32),
    EqualTo(FollowVariable, f32),
}
// We can do a naive custom approach:
impl PartialEq for ComparisonNF {
    fn eq(&self, other: &Self) -> bool {
        // compare both the enum discriminant and the float bits
        use ComparisonNF::*;
        match (self, other) {
            (GreaterThan(a1, f1), GreaterThan(a2, f2)) => a1 == a2 && f1.to_bits() == f2.to_bits(),
            (LessThan(a1, f1), LessThan(a2, f2)) => a1 == a2 && f1.to_bits() == f2.to_bits(),
            (EqualTo(a1, f1), EqualTo(a2, f2)) => a1 == a2 && f1.to_bits() == f2.to_bits(),
            _ => false,
        }
    }
}

impl Eq for ComparisonNF {}

// A "fake" total ordering that compares float bits, ignoring NaNs or special cases
use std::cmp::Ordering;

impl PartialOrd for ComparisonNF {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for ComparisonNF {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for ComparisonNF {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use ComparisonNF::*;
        std::mem::discriminant(self).hash(state);
        match self {
            GreaterThan(a, x) | LessThan(a, x) | EqualTo(a, x) => {
                a.hash(state);
                x.to_bits().hash(state);
            }
        }
    }
}

/// NF for actions
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct ActionNF {
    pub transform_f: FollowTransformFn,
    pub transform_g: FollowTransformFn,
}

impl Default for ActionNF {
    fn default() -> Self {
        ActionNF {
            transform_f: FollowTransformFn::new(|_, _| 1.0),
            transform_g: FollowTransformFn::new(|_, _| 1.0),
        }
    }
}
