#[cfg(test)]
mod tests {

    use crate::follow::types::*;
    use num_rational::Rational64;

    #[test]
    fn test_single_conditional_rule() {
        let program = Program {
            follows: vec![Follow {
                rules: vec![FollowRule::Conditional {
                    condition: FollowCondition::And(FollowAndCondition::Simple(
                        FollowSimpleCondition::Comparison(FollowComparison::GreaterThan(
                            FollowVariable::F,
                            Rational64::new(2, 1),
                        )),
                    )),
                    action: FollowAction {
                        expr_f: FollowExpr::Variable(FollowVariable::F),
                        expr_g: FollowExpr::Value(Rational64::new(4, 1)),
                    },
                }],
            }],
        };

        let expected = FollowNF::Condition {
            test: ConditionNF::Comparison(ComparisonNF::GreaterThan(FollowVariable::F, 2.0)),
            true_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|f, _| f),
                transform_g: FollowTransformFn::new(|_, _| 4.0),
            })),
            false_branch: Box::new(FollowNF::Leaf(ActionNF::default())),
        };

        let default_action = ActionNF::default();
        let compiled = program.to_nf(Some(default_action.clone()));

        assert_eq!(compiled[0], expected);
    }

    #[test]
    fn test_default_rule() {
        let program = Program {
            follows: vec![Follow {
                rules: vec![FollowRule::Default(FollowAction {
                    expr_f: FollowExpr::Value(Rational64::new(0, 1)),
                    expr_g: FollowExpr::Value(Rational64::new(0, 1)),
                })],
            }],
        };

        let expected = FollowNF::Leaf(ActionNF {
            transform_f: FollowTransformFn::new(|_, _| 0.0),
            transform_g: FollowTransformFn::new(|_, _| 0.0),
        });

        let default_action = ActionNF::default();
        let compiled = program.to_nf(Some(default_action.clone()));

        assert_eq!(compiled[0], expected);
    }

    #[test]
    fn test_multiple_rules() {
        let program = Program {
            follows: vec![Follow {
                rules: vec![
                    FollowRule::Conditional {
                        condition: FollowCondition::And(FollowAndCondition::Simple(
                            FollowSimpleCondition::Comparison(FollowComparison::GreaterThan(
                                FollowVariable::F,
                                Rational64::new(2, 1),
                            )),
                        )),
                        action: FollowAction {
                            expr_f: FollowExpr::Variable(FollowVariable::F),
                            expr_g: FollowExpr::Value(Rational64::new(4, 1)),
                        },
                    },
                    FollowRule::Default(FollowAction {
                        expr_f: FollowExpr::Value(Rational64::new(0, 1)),
                        expr_g: FollowExpr::Value(Rational64::new(0, 1)),
                    }),
                ],
            }],
        };

        let expected = FollowNF::Condition {
            test: ConditionNF::Comparison(ComparisonNF::GreaterThan(FollowVariable::F, 2.0)),
            true_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|f, _| f),
                transform_g: FollowTransformFn::new(|_, _| 4.0),
            })),
            false_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|_, _| 0.0),
                transform_g: FollowTransformFn::new(|_, _| 0.0),
            })),
        };

        let default_action = ActionNF::default();
        let compiled = program.to_nf(Some(default_action.clone()));

        assert_eq!(compiled[0], expected);
    }

    #[test]
    fn test_and() {
        let program = Program {
            follows: vec![Follow {
                rules: vec![
                    FollowRule::Conditional {
                        condition: FollowCondition::And(FollowAndCondition::And(
                            Box::new(FollowAndCondition::Simple(
                                FollowSimpleCondition::Comparison(FollowComparison::GreaterThan(
                                    FollowVariable::F,
                                    Rational64::new(2, 1),
                                )),
                            )),
                            Box::new(FollowSimpleCondition::Comparison(
                                FollowComparison::LessThan(
                                    FollowVariable::G,
                                    Rational64::new(3, 1),
                                ),
                            )),
                        )),
                        action: FollowAction {
                            expr_f: FollowExpr::Variable(FollowVariable::F),
                            expr_g: FollowExpr::Value(Rational64::new(4, 1)),
                        },
                    },
                    FollowRule::Default(FollowAction {
                        expr_f: FollowExpr::Value(Rational64::new(0, 1)),
                        expr_g: FollowExpr::Value(Rational64::new(0, 1)),
                    }),
                ],
            }],
        };

        let expected = FollowNF::Condition {
            test: ConditionNF::LogicalAnd(
                Box::new(ConditionNF::Comparison(ComparisonNF::GreaterThan(
                    FollowVariable::F,
                    2.0,
                ))),
                Box::new(ConditionNF::Comparison(ComparisonNF::LessThan(
                    FollowVariable::G,
                    3.0,
                ))),
            ),
            true_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|f, _| f),
                transform_g: FollowTransformFn::new(|_, _| 4.0),
            })),
            false_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|_, _| 0.0),
                transform_g: FollowTransformFn::new(|_, _| 0.0),
            })),
        };

        let default_action = ActionNF::default();
        let compiled = program.to_nf(Some(default_action.clone()));

        assert_eq!(compiled[0], expected);
    }

    #[test]
    fn test_or() {
        let program = Program {
            follows: vec![Follow {
                rules: vec![
                    FollowRule::Conditional {
                        condition: FollowCondition::Or(
                            Box::new(FollowCondition::And(FollowAndCondition::Simple(
                                FollowSimpleCondition::Comparison(FollowComparison::GreaterThan(
                                    FollowVariable::F,
                                    Rational64::new(2, 1),
                                )),
                            ))),
                            Box::new(FollowAndCondition::Simple(
                                FollowSimpleCondition::Comparison(FollowComparison::LessThan(
                                    FollowVariable::G,
                                    Rational64::new(3, 1),
                                )),
                            )),
                        ),
                        action: FollowAction {
                            expr_f: FollowExpr::Variable(FollowVariable::F),
                            expr_g: FollowExpr::Value(Rational64::new(4, 1)),
                        },
                    },
                    FollowRule::Default(FollowAction {
                        expr_f: FollowExpr::Value(Rational64::new(0, 1)),
                        expr_g: FollowExpr::Value(Rational64::new(0, 1)),
                    }),
                ],
            }],
        };

        let expected = FollowNF::Condition {
            test: ConditionNF::LogicalOr(
                Box::new(ConditionNF::Comparison(ComparisonNF::GreaterThan(
                    FollowVariable::F,
                    2.0,
                ))),
                Box::new(ConditionNF::Comparison(ComparisonNF::LessThan(
                    FollowVariable::G,
                    3.0,
                ))),
            ),
            true_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|f, _| f),
                transform_g: FollowTransformFn::new(|_, _| 4.0),
            })),
            false_branch: Box::new(FollowNF::Leaf(ActionNF {
                transform_f: FollowTransformFn::new(|_, _| 0.0),
                transform_g: FollowTransformFn::new(|_, _| 0.0),
            })),
        };

        let default_action = ActionNF::default();
        let compiled = program.to_nf(Some(default_action.clone()));

        assert_eq!(compiled[0], expected);
    }
}
