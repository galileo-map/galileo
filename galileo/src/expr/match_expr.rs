use serde::{Deserialize, Serialize};

use crate::expr::{Expr, ExprFeature, ExprValue, ExprView};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpr {
    pub input: Expr,
    pub cases: Vec<MatchCase>,
    pub fallback: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub in_values: Vec<ExprValue<String>>,
    pub out: Expr,
}

impl MatchExpr {
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<&'a str> {
        let input = self.input.eval(f, v);
        let fallback = self.fallback.eval(f, v);
        if input.is_null() {
            return fallback;
        }

        for case in &self.cases {
            for in_value in &case.in_values {
                if in_value.borrowed() == input {
                    return case.out.eval(f, v);
                }
            }
        }

        fallback
    }
}
