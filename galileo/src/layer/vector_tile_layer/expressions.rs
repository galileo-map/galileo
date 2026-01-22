//! Module that provides support for step and interpolate expressions for Color and Number
//! type expression
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::vec::IntoIter;

use serde::{Deserialize, Serialize};

use crate::Color;
/// Wrapper over arguments for Interpolation Functions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InterpolationArgs<T> {
    /// Linear variant of interpolation arguments
    #[serde(rename = "linear")]
    Linear(LinearInterpolationArgs<T>),
    /// Exponential variant of interpolation arguments
    #[serde(rename = "exponential")]
    Exponential(ExponentialInterpolationArgs<T>),
    /// Cubic variant of interpolation arguments
    #[serde(rename = "cubic")]
    Cubic(CubicInterpolationArgs<T>),
}

impl<T: Copy> InterpolationArgs<T> {
    fn step_values(&self) -> &BTreeSet<StepValue<T>> {
        match self {
            Self::Linear(args) => &args.step_values,
            Self::Exponential(args) => &args.step_values,
            Self::Cubic(args) => &args.step_values,
        }
    }
}
/// Arguments for Linear Interpolation Function
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearInterpolationArgs<T> {
    step_values: BTreeSet<StepValue<T>>,
}

impl<T: Copy> LinearInterpolationArgs<T> {
    /// Returns a new instance of `LinearInterpolationArgs`
    pub fn new(step_values: IntoIter<StepValue<T>>) -> Result<Self, String> {
        if step_values.len() < 2 {
            return Err("At least 2 step values required".to_string());
        }
        Ok(Self {
            step_values: step_values.collect::<BTreeSet<_>>(),
        })
    }
}

/// Arguments for Exponential Interpolation Function
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct ExponentialInterpolationArgs<T> {
    base: i32,
    step_values: BTreeSet<StepValue<T>>,
}

impl<T: Copy> ExponentialInterpolationArgs<T> {
    /// Returns a new instance of `ExponentialInterpolationArgs`
    pub fn new(base: i32, step_values: IntoIter<StepValue<T>>) -> Result<Self, String> {
        if base <= 0 {
            return Err("Base must be positive".to_string());
        }

        if step_values.len() < 2 {
            return Err("At least 2 step values required".to_string());
        }
        Ok(Self {
            base,
            step_values: step_values.into_iter().collect::<BTreeSet<_>>(),
        })
    }
}

/// Arguments for Cubic Interpolation Function
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct CubicInterpolationArgs<T> {
    control_points: [f64; 4],
    step_values: BTreeSet<StepValue<T>>,
}

impl<T: Copy> CubicInterpolationArgs<T> {
    /// Returns a new instance of `CubicInterpolationArgs`
    pub fn new(
        control_points: [f64; 4],
        step_values: IntoIter<StepValue<T>>,
    ) -> Result<Self, String> {
        if step_values.len() < 2 {
            return Err("At least 2 step values required".to_string());
        }
        Ok(Self {
            control_points,
            step_values: step_values.into_iter().collect::<BTreeSet<_>>(),
        })
    }
}

/// Wrapper type for each step value
/// i.e. resolution and a Color or a Number
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StepValue<T> {
    /// Minimum resolution for the step to be used.
    pub resolution: f64,
    /// Literal value for the step..
    pub step_value: T,
}

impl<T> PartialEq for StepValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.resolution == other.resolution
    }
}

impl<T> Eq for StepValue<T> {}

impl<T> PartialOrd for StepValue<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for StepValue<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.resolution.total_cmp(&other.resolution)
    }
}

/// Type used to define expressions for interpolation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InterpolateExpression<T> {
    #[serde(rename = "interpolate")]
    interpolation_args: InterpolationArgs<T>,
}
/// Type used to define Step Function
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StepExpression<T> {
    /// Each stop value maps the resolution to the T type
    /// If, the current resolution is greater than step resolution
    /// the value T maps to the T value where the step resolution is
    /// less than that of current resolution.
    #[serde(rename = "default_value")]
    default_value: T,
    step_values: BTreeSet<StepValue<T>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ResolutionValueRange<T> {
    max_resolution: f64,
    min_resolution: f64,
    start_value: T,
    end_value: T,
}

/// StyleValue introduces simple, interpolation and step functions to be used as features that are
/// evaluated to give color values on the basis of the zoom level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StyleValue<T> {
    /// Style variant to wrap interpolate function
    Interpolate(InterpolateExpression<T>),
    /// Style variant to wrap step function
    Steps(StepExpression<T>),
    /// Style variant simple values
    Simple(T),
}

impl<T> From<T> for StyleValue<T> {
    fn from(value: T) -> Self {
        Self::Simple(value)
    }
}

impl StyleValue<Color> {
    /// Evaluates value of Color depending upon the type of expression used.
    pub fn get_value(&self, current_resolution: f64) -> Color {
        match self {
            StyleValue::Simple(t) => *t,
            StyleValue::Interpolate(expression) => expression.evaluate(current_resolution),
            StyleValue::Steps(expression) => expression.evaluate(current_resolution),
        }
    }
}
impl StyleValue<f64> {
    /// Evaluates value of Number depending upon the type of expression used.
    pub fn get_value(&self, current_resolution: f64) -> f64 {
        match self {
            StyleValue::Simple(t) => *t,
            StyleValue::Interpolate(expression) => expression.evaluate(current_resolution),
            StyleValue::Steps(expression) => expression.evaluate(current_resolution),
        }
    }
}

impl StyleValue<f32> {
    /// Evaluates value of Number depending upon the type of expression used.
    pub fn get_value(&self, current_resolution: f64) -> f32 {
        match self {
            StyleValue::Simple(t) => *t,
            StyleValue::Interpolate(expression) => expression.evaluate(current_resolution),
            StyleValue::Steps(expression) => expression.evaluate(current_resolution),
        }
    }
}

impl<T> InterpolateExpression<T> {
    /// Returns a new instance of `InterpolateExpression`
    pub fn new(interpolation_args: InterpolationArgs<T>) -> Self {
        Self { interpolation_args }
    }
    fn get_boundary_value(&self, current_resolution: f64) -> T
    where
        T: Copy,
    {
        let step_values = self
            .interpolation_args
            .step_values()
            .iter()
            .collect::<Vec<_>>();

        if current_resolution < step_values[0].resolution {
            step_values[0].step_value
        } else {
            step_values[step_values.len() - 1].step_value
        }
    }
}

pub(crate) enum Channel {
    R,
    G,
    B,
    A,
}
/// This trait is used to define interpolatability for a type
/// on implementing this, `InterpolateExpression` allows for the use of evaluate method.
trait InterpolatableValue: Copy {
    /// Value to be interpolated
    type COMPONENTS;

    /// Returns the Iterator for the components used
    fn iter_components() -> impl Iterator<Item = Self::COMPONENTS>;
    /// Returns the Iterator for the components used
    fn get_component(&self, component: &Self::COMPONENTS) -> f64;
    /// Value to be interpolated
    fn set_component(&mut self, component: &Self::COMPONENTS, value: f64);
}

impl InterpolatableValue for f32 {
    type COMPONENTS = ();

    fn iter_components() -> impl Iterator<Item = Self::COMPONENTS> {
        std::iter::once(())
    }

    fn get_component(&self, _component: &Self::COMPONENTS) -> f64 {
        *self as f64
    }

    fn set_component(&mut self, _component: &Self::COMPONENTS, value: f64) {
        *self = value as f32
    }
}

impl InterpolatableValue for f64 {
    type COMPONENTS = ();

    fn iter_components() -> impl Iterator<Item = Self::COMPONENTS> {
        std::iter::once(())
    }

    fn get_component(&self, _component: &Self::COMPONENTS) -> f64 {
        *self
    }

    fn set_component(&mut self, _component: &Self::COMPONENTS, value: f64) {
        *self = value
    }
}

impl InterpolatableValue for Color {
    type COMPONENTS = Channel;

    fn iter_components() -> impl Iterator<Item = Self::COMPONENTS> {
        [Channel::R, Channel::G, Channel::B, Channel::A].into_iter()
    }

    fn get_component(&self, channel: &Self::COMPONENTS) -> f64 {
        let val = match channel {
            Channel::R => self.r(),
            Channel::G => self.g(),
            Channel::B => self.b(),
            Channel::A => self.a(),
        };

        val as f64
    }

    fn set_component(&mut self, channel: &Self::COMPONENTS, value: f64) {
        let val = value as u8;
        let [mut r, mut g, mut b, mut a] = self.to_u8_array();
        match channel {
            Channel::R => r = val,
            Channel::G => g = val,
            Channel::B => b = val,
            Channel::A => a = val,
        }

        *self = Color::rgba(r, g, b, a)
    }
}
#[allow(private_bounds)]
impl<T: InterpolatableValue> InterpolateExpression<T> {
    fn evaluate(&self, current_resolution: f64) -> T {
        if let Some(resolution_value_range) = get_resolution_value_range(self, current_resolution) {
            self.interpolate_value(current_resolution, &resolution_value_range)
        } else {
            self.get_boundary_value(current_resolution)
        }
    }

    fn interpolate_value(&self, current_resolution: f64, rv_range: &ResolutionValueRange<T>) -> T {
        let mut result = rv_range.start_value;

        for component in T::iter_components() {
            let start = rv_range.start_value.get_component(&component);
            let end = rv_range.end_value.get_component(&component);

            let component_value = match &self.interpolation_args {
                InterpolationArgs::Linear(_) => linear_interpolation(
                    rv_range.min_resolution,
                    rv_range.max_resolution,
                    start,
                    end,
                    current_resolution,
                ),
                InterpolationArgs::Exponential(args) => exponential_interpolation(
                    rv_range.min_resolution,
                    rv_range.max_resolution,
                    start,
                    end,
                    current_resolution,
                    args.base,
                ),
                InterpolationArgs::Cubic(args) => cubic_interpolation(
                    rv_range.min_resolution,
                    rv_range.max_resolution,
                    start,
                    end,
                    current_resolution,
                    args.control_points,
                ),
            };

            result.set_component(&component, component_value);
        }

        result
    }
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

fn inv_bezier(x0: f64, cpts: [f64; 4]) -> f64 {
    let x1 = cpts[0];
    let x2 = cpts[2];
    // Bx(t)
    let f = move |t: f64| {
        3. * (1. - t).powi(2) * t * x1 + 3. * (1. - t) * t.powi(2) * x2 + t.powi(3) - x0
    };
    bisection_solve(f, 0., 1., 0.001)
}

///  Cubic bezier interpolation solver.
///
/// The math formula for algorithm is:
///   - `Bx(t) = 3(1-t)²t·x1 + 3(1-t)t²·x2 + t³`
///   - `By(t) = 3(1-t)²t·y1 + 3(1-t)t²·y2 + t³`
///     Implementation is taken from: https://en.wikipedia.org/wiki/B%C3%A9zier_curve
///
fn cubic_interpolation(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    x0: f64,
    control_points: [f64; 4],
) -> f64 {
    let x_normalized =
        ((x0 - x_start) / (x_end - x_start).clamp(f64::EPSILON, f64::MAX)).clamp(0., 1.);
    // inverse of Bx(t) for x_normalized
    let t = inv_bezier(x_normalized, control_points);
    let y1 = control_points[1];
    let y2 = control_points[3];
    // By(t)
    let y_normalized = 3. * (1. - t).powi(2) * t * y1 + 3. * (1. - t) * t.powi(2) * y2 + t.powi(3);
    y_start + y_normalized * (y_end - y_start)
}

fn exponential_interpolation(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    x0: f64,
    base: i32,
) -> f64 {
    let t: f64 = ((x0 - x_start) / (x_end - x_start).clamp(f64::EPSILON, f64::MAX)).clamp(0.0, 1.0);

    let base: f64 = base as f64;
    let t = if (base - 1.0).abs() > f64::EPSILON {
        (base.powf(t) - 1.0) / (base - 1.0)
    } else {
        t
    };

    let offset = y_end - y_start;

    y_start + t * (offset)
}

fn linear_interpolation(x_start: f64, x_end: f64, y_start: f64, y_end: f64, x0: f64) -> f64 {
    let x_range: f64 = (x_end - x_start).clamp(f64::EPSILON, f64::MAX);

    let k = (y_end - y_start) / x_range;

    let offset = (x0 - x_start).clamp(0.0, x_range);
    y_start + k * offset
}

fn get_resolution_value_range<T: Copy>(
    expression: &InterpolateExpression<T>,
    current_resolution: f64,
) -> Option<ResolutionValueRange<T>> {
    let step_values: &BTreeSet<StepValue<T>> = expression.interpolation_args.step_values();
    // Try to find a matching window in step_values
    step_values
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| current_resolution >= w[0].resolution && current_resolution <= w[1].resolution)
        .map(|w| ResolutionValueRange {
            min_resolution: w[0].resolution,
            max_resolution: w[1].resolution,
            start_value: w[0].step_value,
            end_value: w[1].step_value,
        })
}

impl<T: Copy> StepExpression<T> {
    /// Returns a new instance of `StepExpression`
    pub fn new(default_value: T, step_values: IntoIter<StepValue<T>>) -> Result<Self, String> {
        if step_values.len() == 0 {
            return Err("At least 1 step value required".to_string());
        }
        Ok(Self {
            default_value,
            step_values: step_values.collect::<BTreeSet<_>>(),
        })
    }
}
impl<T: Copy> StepExpression<T> {
    /// Evaluates generic expression by giving stepwise value
    /// of color on basis of zoom
    fn evaluate(&self, current_resolution: f64) -> T {
        if let Some(w) = self
            .step_values
            .iter()
            .collect::<Vec<_>>()
            .windows(2)
            .find(|w| {
                current_resolution >= w[0].resolution && current_resolution <= w[1].resolution
            })
        {
            w[0].step_value
        } else if current_resolution
            < self
                .step_values
                .iter()
                .nth(0)
                .expect("value at 0th position")
                .resolution
        {
            self.default_value
        } else {
            self.step_values
                .iter()
                .nth(self.step_values.len() - 1)
                .expect("value at end position")
                .step_value
        }
    }
}

#[cfg(test)]
mod number_tests {
    use super::*;

    #[test]
    fn test_get_resolution_value_range_out_of_bounds_f64() {
        let args: LinearInterpolationArgs<f64> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 25.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression {
            interpolation_args: InterpolationArgs::Linear(args),
        };

        assert!(get_resolution_value_range(&expr, 75.0).is_none());
        assert!(get_resolution_value_range(&expr, 20.0).is_none());
    }

    #[test]
    fn linear_interpolation_bounds_f64() {
        let args: LinearInterpolationArgs<f64> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 25.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(20.0).round(), 0.0);
        assert_eq!(expr.evaluate(150.0).round(), 100.0);
    }

    #[test]
    fn linear_interpolation_unordered_f64() {
        let args: LinearInterpolationArgs<f64> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(25.0).round(), 50.0);
    }

    #[test]
    fn linear_interpolation_f64() {
        let args: LinearInterpolationArgs<f64> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(25.0).round(), 50.0);
    }

    #[test]
    fn exponential_bounds_f64() {
        let args: ExponentialInterpolationArgs<f64> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 10.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(5.0).round(), 0.0);
        assert_eq!(expr.evaluate(150.0).round(), 100.0);
    }

    #[test]
    fn exponential_interpolation_unordered_f64() {
        let args: ExponentialInterpolationArgs<f64> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(25.0).round(), 41.0);
        assert_eq!(expr.evaluate(60.0).round(), 132.0);
    }

    #[test]
    fn exponential_interpolation_f64() {
        let args: ExponentialInterpolationArgs<f64> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(25.0).round(), 41.0);
        assert_eq!(expr.evaluate(60.0).round(), 132.0);
    }

    #[test]
    fn cubic_interpolation_bounds() {
        let args: CubicInterpolationArgs<Color> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(0.0), Color::rgba(128, 128, 128, 128));
        assert_eq!(expr.evaluate(100.0), Color::rgba(200, 200, 200, 200));
    }

    #[test]
    fn cubic_interpolation_bounds_f64() {
        let args: CubicInterpolationArgs<Color> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(0.0), Color::rgba(128, 128, 128, 128));
        assert_eq!(expr.evaluate(100.0), Color::rgba(200, 200, 200, 200));
    }

    #[test]
    fn cubic_interpolation() {
        let args: CubicInterpolationArgs<Color> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(108, 108, 108, 108));
        assert_eq!(expr.evaluate(60.0), Color::rgba(186, 186, 186, 186));
    }

    #[test]
    fn cubic_interpolation_f64() {
        let args: CubicInterpolationArgs<f64> = CubicInterpolationArgs::new(
            [0.0, 0.25, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0).round(), 67.0);
        assert_eq!(expr.evaluate(60.0).round(), 158.0);
    }

    #[test]
    fn cubic_interpolation_unordered() {
        let args: CubicInterpolationArgs<Color> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(108, 108, 108, 108));
        assert_eq!(expr.evaluate(60.0), Color::rgba(186, 186, 186, 186));
    }

    #[test]
    fn cubic_interpolation_unordered_f64() {
        let args: CubicInterpolationArgs<f64> = CubicInterpolationArgs::new(
            [0.0, 0.25, 0.5, 0.75],
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0).round(), 67.0);
        assert_eq!(expr.evaluate(60.0).round(), 158.0);
    }

    #[test]
    fn cubic_interpolation_symmetric_control_points() {
        let args: CubicInterpolationArgs<Color> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.0, 1.0],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(126, 126, 126, 126));
        assert_eq!(expr.evaluate(60.0), Color::rgba(198, 198, 198, 198));
    }

    #[test]
    fn cubic_interpolation_symmetric_control_points_f64() {
        let args: CubicInterpolationArgs<f64> = CubicInterpolationArgs::new(
            [0.0, 1.0, 0.0, 1.0],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0).round(), 99.0);
        assert_eq!(expr.evaluate(60.0).round(), 198.0);
    }

    #[test]
    fn cubic_interpolation_zeroes_control_points() {
        let args: CubicInterpolationArgs<f64> = CubicInterpolationArgs::new(
            [0.0, 0.0, 0.0, 0.0],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0).round(), 50.0);
        assert_eq!(expr.evaluate(60.0).round(), 140.0);
    }

    #[test]
    fn cubic_interpolation_equal_control_points() {
        let args: CubicInterpolationArgs<f64> = CubicInterpolationArgs::new(
            [0.3, 0.3, 0.3, 0.3],
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: 0.0,
                },
                StepValue {
                    resolution: 50.0,
                    step_value: 100.0,
                },
                StepValue {
                    resolution: 75.0,
                    step_value: 200.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Cubic(args));

        assert_eq!(expr.evaluate(25.0).round(), 50.0);
        assert_eq!(expr.evaluate(60.0).round(), 140.0);
    }

    #[test]
    fn test_step_expression_bounds_f64() {
        let expr: StepExpression<f64> = StepExpression::new(
            10.0,
            vec![
                StepValue {
                    resolution: 10.0,
                    step_value: 20.0,
                },
                StepValue {
                    resolution: 20.0,
                    step_value: 30.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create step expression");

        assert_eq!(expr.evaluate(5.0).round(), 10.0);
        assert_eq!(expr.evaluate(30.0).round(), 30.0);
    }

    #[test]
    fn test_step_expression_f64() {
        let expr: StepExpression<f64> = StepExpression::new(
            10.0,
            vec![
                StepValue {
                    resolution: 10.0,
                    step_value: 20.0,
                },
                StepValue {
                    resolution: 20.0,
                    step_value: 30.0,
                },
                StepValue {
                    resolution: 30.0,
                    step_value: 40.0,
                },
            ]
            .into_iter(),
        )
        .expect("failed to create step expression");

        assert_eq!(expr.evaluate(15.0).round(), 20.0);
        assert_eq!(expr.evaluate(25.0).round(), 30.0);
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn test_get_resolution_value_range_out_of_bounds() {
        let args: LinearInterpolationArgs<Color> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 25.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression {
            interpolation_args: InterpolationArgs::Linear(args),
        };

        assert!(get_resolution_value_range(&expr, 75.0).is_none());
        assert!(get_resolution_value_range(&expr, 20.0).is_none());
    }

    #[test]
    fn linear_interpolation_bounds() {
        let args: LinearInterpolationArgs<Color> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 25.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(20.0), Color::rgba(0, 0, 0, 0));
        assert_eq!(expr.evaluate(150.0), Color::rgba(128, 128, 128, 128));
    }

    #[test]
    fn linear_interpolation_unordered() {
        let args: LinearInterpolationArgs<Color> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(64, 64, 64, 64));
    }

    #[test]
    fn linear_interpolation() {
        let args: LinearInterpolationArgs<Color> = LinearInterpolationArgs::new(
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Linear(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(64, 64, 64, 64));
    }

    #[test]
    fn exponential_bounds() {
        let args: ExponentialInterpolationArgs<Color> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 10.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(5.0), Color::rgba(0, 0, 0, 0));
        assert_eq!(expr.evaluate(150.0), Color::rgba(128, 128, 128, 128));
    }

    #[test]
    fn exponential_interpolation_unordered() {
        let args: ExponentialInterpolationArgs<Color> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(53, 53, 53, 53));
        assert_eq!(expr.evaluate(60.0), Color::rgba(151, 151, 151, 151));
    }

    #[test]
    fn exponential_interpolation() {
        let args: ExponentialInterpolationArgs<Color> = ExponentialInterpolationArgs::new(
            2,
            vec![
                StepValue {
                    resolution: 0.0,
                    step_value: Color::rgba(0, 0, 0, 0),
                },
                StepValue {
                    resolution: 50.0,
                    step_value: Color::rgba(128, 128, 128, 128),
                },
                StepValue {
                    resolution: 75.0,
                    step_value: Color::rgba(200, 200, 200, 200),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create interpolation arguments");

        let expr = InterpolateExpression::new(InterpolationArgs::Exponential(args));

        assert_eq!(expr.evaluate(25.0), Color::rgba(53, 53, 53, 53));
        assert_eq!(expr.evaluate(60.0), Color::rgba(151, 151, 151, 151));
    }

    #[test]
    fn test_step_expression_bounds() {
        let expr = StepExpression::<Color>::new(
            Color::from_hex("#f0f0f0"),
            vec![
                StepValue::<Color> {
                    resolution: 10.0,
                    step_value: Color::from_hex("#fafafa"),
                },
                StepValue::<Color> {
                    resolution: 20.0,
                    step_value: Color::from_hex("#1d1d1d"),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create step expression");

        assert_eq!(expr.evaluate(5.0), Color::from_hex("#f0f0f0"));
        assert_eq!(expr.evaluate(30.0), Color::from_hex("#1d1d1d"));
    }

    #[test]
    fn test_step_expression() {
        let expr = StepExpression::<Color>::new(
            Color::from_hex("#f0f0f0"),
            vec![
                StepValue::<Color> {
                    resolution: 10.0,
                    step_value: Color::from_hex("#fafafa"),
                },
                StepValue::<Color> {
                    resolution: 20.0,
                    step_value: Color::from_hex("#1d1d1d"),
                },
                StepValue::<Color> {
                    resolution: 30.0,
                    step_value: Color::from_hex("#1a1a1a"),
                },
            ]
            .into_iter(),
        )
        .expect("failed to create step expression");

        assert_eq!(expr.evaluate(15.0), Color::from_hex("#fafafa"));
        assert_eq!(expr.evaluate(25.0), Color::from_hex("#1d1d1d"));
    }
}
