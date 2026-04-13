use std::collections::BTreeMap;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::Color;
use crate::expr::{Expr, ExprFeature, ExprValue, ExprView};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlPoint {
    pub input: Expr,
    pub output: Expr,
}

pub trait Interpolation {
    fn input(&self) -> &Expr;
    fn control_points(&self) -> &[ControlPoint];
    fn interpolate_num(&self, t: f64, x1: f64, out1: f64, x2: f64, out2: f64) -> f64;

    fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<'static> {
        let Some(input) = self.input().eval(f, v).as_number() else {
            return ExprValue::Null;
        };

        if !input.is_finite() {
            return ExprValue::Null;
        }

        let Some(control_points) = self.eval_control_points(f, v) else {
            return ExprValue::Null;
        };

        let input = OrderedFloat::from(input);
        let lower = control_points.range(..input).next_back();
        let upper = control_points.range(input..).next();

        match (lower, upper) {
            (None, None) => ExprValue::Null,
            (Some(lower), None) => lower.1.eval(f, v).owned(),
            (None, Some(upper)) => upper.1.eval(f, v).owned(),
            (Some(lower), Some(upper)) => self.interpolate(
                *input,
                **lower.0,
                lower.1.eval(f, v),
                **upper.0,
                upper.1.eval(f, v),
            ),
        }
    }

    fn eval_control_points<'a>(
        &'a self,
        f: &impl ExprFeature,
        v: ExprView,
    ) -> Option<BTreeMap<OrderedFloat<f64>, &'a Expr>> {
        let mut evaluated = BTreeMap::new();
        for point in self.control_points() {
            let value = point.input.eval(f, v).as_number()?;
            if !value.is_finite() {
                return None;
            }

            evaluated.insert(value.into(), &point.output);
        }

        Some(evaluated)
    }

    fn interpolate(
        &self,
        t: f64,
        x1: f64,
        out1: ExprValue<'_>,
        x2: f64,
        out2: ExprValue<'_>,
    ) -> ExprValue<'static> {
        match (out1, out2) {
            (ExprValue::Number(out1), ExprValue::Number(out2)) => {
                self.interpolate_num(t, x1, out1, x2, out2).into()
            }
            (ExprValue::Color(out1), ExprValue::Color(out2)) => {
                self.interpolate_color(t, x1, out1, x2, out2).into()
            }
            (ExprValue::NumArray(out1), ExprValue::NumArray(out2)) => self
                .interpolate_num_array(t, x1, &out1, x2, &out2)
                .map(|v| v.into())
                .unwrap_or(ExprValue::Null),
            _ => ExprValue::Null,
        }
    }

    fn interpolate_color(&self, t: f64, x1: f64, out1: Color, x2: f64, out2: Color) -> Color {
        let r = self.interpolate_num(t, x1, out1.r() as f64, x2, out2.r() as f64) as u8;
        let g = self.interpolate_num(t, x1, out1.g() as f64, x2, out2.g() as f64) as u8;
        let b = self.interpolate_num(t, x1, out1.b() as f64, x2, out2.b() as f64) as u8;
        let a = self.interpolate_num(t, x1, out1.a() as f64, x2, out2.a() as f64) as u8;

        Color::rgba(r, g, b, a)
    }

    fn interpolate_num_array(
        &self,
        t: f64,
        x1: f64,
        out1: &[f64],
        x2: f64,
        out2: &[f64],
    ) -> Option<Vec<f64>> {
        if out1.is_empty() || out1.len() != out2.len() {
            return None;
        }

        let mut result = vec![0.0; out1.len()];
        for i in 0..out1.len() {
            result[i] = self.interpolate_num(t, x1, out1[i], x2, out2[i]);
        }

        Some(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearInterpolation {
    pub input: Expr,
    pub control_points: Vec<ControlPoint>,
}

impl Interpolation for LinearInterpolation {
    fn input(&self) -> &Expr {
        &self.input
    }

    fn control_points(&self) -> &[ControlPoint] {
        &self.control_points
    }

    fn interpolate_num(&self, t: f64, x1: f64, out1: f64, x2: f64, out2: f64) -> f64 {
        let x_range: f64 = (x2 - x1).clamp(f64::EPSILON, f64::MAX);

        let k = (out2 - out1) / x_range;

        let offset = (t - x1).clamp(0.0, x_range);
        out1 + k * offset
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExponentialInterpolation {
    pub base: f64,
    pub input: Expr,
    pub control_points: Vec<ControlPoint>,
}

impl Interpolation for ExponentialInterpolation {
    fn input(&self) -> &Expr {
        &self.input
    }

    fn control_points(&self) -> &[ControlPoint] {
        &self.control_points
    }

    fn interpolate_num(&self, x0: f64, x_start: f64, y_start: f64, x_end: f64, y_end: f64) -> f64 {
        let base = self.base;
        let t: f64 =
            ((x0 - x_start) / (x_end - x_start).clamp(f64::EPSILON, f64::MAX)).clamp(0.0, 1.0);

        let t = if (base - 1.0).abs() > f64::EPSILON {
            (base.powf(t) - 1.0) / (base - 1.0)
        } else {
            t
        };

        let offset = y_end - y_start;

        y_start + t * (offset)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicBezierInterpolation {
    pub input: Expr,
    pub control_points: Vec<ControlPoint>,
    pub curve_params: [f64; 4],
}

impl Interpolation for CubicBezierInterpolation {
    fn input(&self) -> &Expr {
        &self.input
    }

    fn control_points(&self) -> &[ControlPoint] {
        &self.control_points
    }

    fn interpolate_num(&self, x0: f64, x_start: f64, y_start: f64, x_end: f64, y_end: f64) -> f64 {
        let control_points = &self.curve_params;
        let x_normalized =
            ((x0 - x_start) / (x_end - x_start).clamp(f64::EPSILON, f64::MAX)).clamp(0., 1.);
        // inverse of Bx(t) for x_normalized
        let t = inv_bezier(x_normalized, control_points);
        let y1 = control_points[1];
        let y2 = control_points[3];
        // By(t)
        let y_normalized =
            3. * (1. - t).powi(2) * t * y1 + 3. * (1. - t) * t.powi(2) * y2 + t.powi(3);
        y_start + y_normalized * (y_end - y_start)
    }
}

fn inv_bezier(x0: f64, cpts: &[f64; 4]) -> f64 {
    let x1 = cpts[0];
    let x2 = cpts[2];
    // Bx(t)
    let f = move |t: f64| {
        3. * (1. - t).powi(2) * t * x1 + 3. * (1. - t) * t.powi(2) * x2 + t.powi(3) - x0
    };
    bisection_solve(f, 0., 1., 0.001)
}

fn bisection_solve(f: impl Fn(f64) -> f64, mut low: f64, mut high: f64, eps: f64) -> f64 {
    let mut t = low;
    const MAX_ITERS: u32 = 25;
    for _ in 0..MAX_ITERS {
        t = low + (high - low) / 2.0;
        let val = f(t);
        if val.abs() < eps {
            return t;
        }
        if val > 0.0 {
            high = t;
        } else {
            low = t;
        }
    }
    t
}
