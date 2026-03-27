#![allow(dead_code)]

use chumsky::error::Rich;
use chumsky::prelude::*;

use crate::Color;
use crate::expr::{Expr, ExprValue};

/// Errors produced by the chumsky-based parser.
pub type ExprParseError = Vec<Rich<'static, char>>;

type Error<'src> = extra::Err<Rich<'src, char>>;

/// Parses a complete expression from a string slice.
pub fn parse_expr(input: &str) -> Result<Expr, ExprParseError> {
    expr_parser()
        .parse(input)
        .into_result()
        .map_err(|errs| errs.into_iter().map(|e| e.into_owned()).collect())
}

fn expr_parser<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>> {
    recursive(|expr_parser| choice((binary(expr_parser.clone()).boxed(), atom(expr_parser))))
}

fn block<'src>(
    expr_parser: impl Parser<'src, &'src str, Expr, Error<'src>> + Clone,
) -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>> + Clone {
    expr_parser
        .delimited_by(just('('), just(')'))
        .labelled("expression block")
}

fn atom<'src>(
    expr_parser: impl Parser<'src, &'src str, Expr, Error<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Expr, Error<'src>> + Clone {
    choice((
        block(expr_parser.clone()),
        literal(),
        func_call(expr_parser),
        property(),
        invalid(),
    ))
    .boxed()
}

fn func0(func_name: &str, constructor: Expr, args: Vec<Expr>) -> Result<Expr, String> {
    if !args.is_empty() {
        return Err(format!(
            "Function `{func_name}` expected 0 arguments, but got {}",
            args.len()
        ));
    }

    Ok(constructor)
}

struct FuncContext<'src> {
    func_name: &'src str,
    func_name_span: SimpleSpan,
    args: Vec<Option<(FuncArgument, &'src str, SimpleSpan)>>,
}

impl<'src> FuncContext<'src> {
    fn new(
        func_name: &'src str,
        func_name_span: SimpleSpan,
        args: Vec<(FuncArgument, &'src str, SimpleSpan)>,
    ) -> Self {
        Self {
            func_name,
            func_name_span,
            args: args.into_iter().map(Some).collect(),
        }
    }

    fn all_args_used(&self, span: SimpleSpan) -> Result<(), Rich<'src, char>> {
        let all_args = self.args.len();
        let unused_args = self.args.iter().filter(|v| v.is_some()).count();

        if unused_args > 0 {
            Err(Rich::custom(
                span,
                format!(
                    "function {} expected {} arguments but got {all_args} instead",
                    self.func_name,
                    all_args - unused_args
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn args1<Arg1>(
        &mut self,
        constructor: impl FnOnce(Arg1) -> Expr,
    ) -> Result<Expr, Rich<'src, char>>
    where
        Self: GetArg<'src, Arg1>,
    {
        Ok(constructor(self.get(0)?))
    }

    fn args2<Arg1, Arg2>(
        &mut self,
        constructor: impl FnOnce(Arg1, Arg2) -> Expr,
    ) -> Result<Expr, Rich<'src, char>>
    where
        Self: GetArg<'src, Arg1> + GetArg<'src, Arg2>,
    {
        Ok(constructor(self.get(0)?, self.get(1)?))
    }

    fn unknown(&self) -> Result<Expr, Rich<'src, char>> {
        Err(Rich::custom(
            self.func_name_span,
            format!("Unknown function `{}`", self.func_name),
        ))
    }

    fn get_arg(
        &mut self,
        index: usize,
    ) -> Result<(FuncArgument, &'src str, SimpleSpan), Rich<'src, char>> {
        if index >= self.args.len() {
            return Err(Rich::custom(
                self.func_name_span,
                format!("function {} missing argumnt {}", self.func_name, index + 1),
            ));
        }

        Ok(self.args[index]
            .take()
            .expect("arguments are used only once"))
    }

    fn arg_type_err(
        &self,
        src: &'src str,
        span: SimpleSpan,
        arg_index: usize,
        expected: &'static str,
    ) -> Rich<'src, char> {
        Rich::custom(
            span,
            format!(
                "expected `{expected}` argument at position `{}` of function `{}`, but got `{src}`",
                arg_index + 1,
                self.func_name,
            ),
        )
    }
}

trait GetArg<'src, T> {
    fn get(&mut self, index: usize) -> Result<T, Rich<'src, char>>;
}

macro_rules! impl_get_arg {
    ($ty:ty, $pat:pat => $val:expr, $expected:literal) => {
        impl<'src> GetArg<'src, $ty> for FuncContext<'src> {
            fn get(&mut self, index: usize) -> Result<$ty, Rich<'src, char>> {
                let arg = self.get_arg(index)?;
                Ok(match arg.0 {
                    $pat => $val,
                    _ => return Err(self.arg_type_err(arg.1, arg.2, index, $expected)),
                })
            }
        }
    };
}

impl_get_arg!(Box<Expr>, FuncArgument::Expression(v) => Box::new(v), "Expression");
impl_get_arg!(
    String,
    FuncArgument::Expression(Expr::Literal(ExprValue::String(v))) => v,
    "String"
);
impl_get_arg!(Vec<Expr>, FuncArgument::Array(v) => v, "[Array]");

#[derive(Debug)]
enum FuncArgument {
    Expression(Expr),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
}

fn func_arg<'src>(
    expr_parser: impl Parser<'src, &'src str, Expr, Error<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, FuncArgument, Error<'src>> {
    let expr = expr_parser.clone().map(FuncArgument::Expression);

    let array = expr_parser
        .clone()
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('['), just(']'))
        .map(FuncArgument::Array);

    let tuple = expr_parser
        .separated_by(just(','))
        .at_least(2)
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('('), just(')'))
        .map(FuncArgument::Tuple);

    choice((array, tuple, expr)).padded()
}

fn func_call<'src>(
    expr_parser: impl Parser<'src, &'src str, Expr, Error<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Expr, Error<'src>> {
    let arg_list = func_arg(expr_parser)
        .map_with(|expr, e| (expr, e.slice(), e.span()))
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('('), just(')'));

    ident()
        .map_with(|func_name, e| (func_name, e.span()))
        .then(arg_list)
        .padded()
        .validate(|((func_name, func_name_span), args), e, emitter| {
            let mut c = FuncContext::new(func_name, func_name_span, args);
            let res = match func_name {
                "any" => c.args1(Expr::Any),
                "all" => c.args1(Expr::All),
                "not" => c.args1(Expr::Not),

                "get" => c.args1(Expr::Get),
                "in" => c.args2(|needle, haystack| Expr::In { needle, haystack }),

                "geom_type" => Ok(Expr::GeomType),
                "zoom" => Ok(Expr::Zoom),
                _ => c.unknown(),
            }
            .and_then(|r| {
                c.all_args_used(e.span())?;
                Ok(r)
            });

            match res {
                Ok(expr) => expr,
                Err(err) => {
                    emitter.emit(err);
                    ExprValue::Null.into()
                }
            }
        })
}

fn property<'src>() -> impl Parser<'src, &'src str, Expr, Error<'src>> {
    ident()
        .padded()
        .map(|name| Expr::Get(name.to_string()))
        .labelled("property name")
}

fn ident<'src>() -> impl Parser<'src, &'src str, &'src str, Error<'src>> {
    any()
        .filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
        .then(ident_cont().repeated())
        .to_slice()
}

fn operator<'src>() -> impl Parser<'src, &'src str, fn(Box<Expr>, Box<Expr>) -> Expr, Error<'src>> {
    choice((
        just("==").to(Expr::Eq as fn(_, _) -> _).labelled("=="),
        just("!=").to(Expr::Ne as fn(_, _) -> _).labelled("!="),
        just(">").to(Expr::Gt as fn(_, _) -> _).labelled(">"),
        just(">=").to(Expr::Gte as fn(_, _) -> _).labelled(">="),
        just("<").to(Expr::Lt as fn(_, _) -> _).labelled("<"),
        just("<=").to(Expr::Lte as fn(_, _) -> _).labelled("<="),
    ))
    .padded()
}

fn binary<'src>(
    expr_parser: impl Parser<'src, &'src str, Expr, Error<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Expr, Error<'src>> {
    atom(expr_parser.clone())
        .then(operator())
        .then(atom(expr_parser))
        .map(|((lhs, constructor), rhs)| constructor(Box::new(lhs), Box::new(rhs)))
}

/// Returns a parser for identifier-continuation characters: ASCII alphanumeric, `_`.
fn ident_cont<'src>() -> impl Parser<'src, &'src str, char, extra::Err<Rich<'src, char>>> {
    any().filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
}

/// Parses `true` or `false`, rejecting a longer identifier that merely starts with those words.
fn bool_literal<'src>()
-> impl Parser<'src, &'src str, ExprValue<String>, extra::Err<Rich<'src, char>>> {
    let true_ = just("true").to(ExprValue::Boolean(true));
    let false_ = just("false").to(ExprValue::Boolean(false));

    choice((true_, false_))
}

/// Parses `null`, rejecting a longer identifier that merely starts with `null`.
fn null_literal<'src>()
-> impl Parser<'src, &'src str, ExprValue<String>, extra::Err<Rich<'src, char>>> {
    just("null").to(ExprValue::Null)
}

/// Parses a hex color literal: `#RRGGBB` or `#RRGGBBAA`.
fn hex_color_literal<'src>()
-> impl Parser<'src, &'src str, ExprValue<String>, extra::Err<Rich<'src, char>>> {
    just('#')
        .ignore_then(
            any()
                .filter(|c: &char| c.is_ascii_hexdigit())
                .repeated()
                .at_least(6)
                .at_most(8)
                .collect::<String>(),
        )
        .try_map(|hex, span| {
            if hex.len() != 6 && hex.len() != 8 {
                return Err(Rich::custom(span, "hex color must be 6 or 8 digits"));
            }
            let color_str = format!("#{hex}");
            Color::try_from_hex(&color_str)
                .map(ExprValue::Color)
                .ok_or_else(|| Rich::custom(span, "invalid hex color"))
        })
}

fn invalid<'src>() -> impl Parser<'src, &'src str, Expr, Error<'src>> {
    any()
        .and_is(atom_boundary().not().rewind())
        .repeated()
        .at_least(1)
        .to_slice()
        .validate(move |s: &str, e, emitter| {
            emitter.emit(Rich::custom(e.span(), format!("invalid value `{s}`")));
            ExprValue::Null.into()
        })
}

fn number_literal<'src>() -> impl Parser<'src, &'src str, ExprValue<String>, Error<'src>> {
    let digits = text::digits(10);

    let fraction = just('.').then(digits.or_not()).or_not();
    let exponent = just('e')
        .or(just('E'))
        .then(just('-').or(just('+')).or_not())
        .then(digits)
        .or_not();

    digits
        .then(fraction)
        .then(exponent)
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map(ExprValue::Number)
                .map_err(|e| Rich::custom(span, e))
        })
}

/// Parses a quoted string literal. Both `"hello"` and `'hello'` are accepted; the opening and
/// closing quote must match. The escape sequences `\"` and `\'` are supported within the body.
fn string_literal<'src>() -> impl Parser<'src, &'src str, String, Error<'src>> {
    let quoted = |quote: char| {
        let escaped = just('\\').ignore_then(just(quote));
        let normal = none_of(['\\', quote]);
        choice((escaped, normal))
            .repeated()
            .collect::<String>()
            .delimited_by(just(quote), just(quote))
    };
    choice((quoted('"'), quoted('\'')))
}

/// Parses any literal value: `bool`, `null`, hex color, number, or quoted string.
fn literal<'src>() -> impl Parser<'src, &'src str, Expr, Error<'src>> {
    choice((
        bool_literal(),
        null_literal(),
        hex_color_literal(),
        number_literal(),
        string_literal().map(ExprValue::from),
    ))
    .padded()
    .map(Expr::Literal)
    .then_ignore(atom_boundary().rewind())
    .labelled("literal")
}

fn atom_boundary<'src>() -> impl Parser<'src, &'src str, (), Error<'src>> {
    choice((operator().ignored(), one_of("()[],").ignored(), end()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(input: &str, expected: Expr) {
        let res = expr_parser().parse(input).into_result();
        assert!(
            res.is_ok(),
            "Errors parsing input `{input}`. Errors: {:?}",
            res.err().unwrap()
        );
        assert_eq!(res.unwrap(), expected, "Wrong parsing result");
    }

    #[test]
    fn parse_atoms() {
        let cases = [
            ("42", 42.0.into()),
            ("42.", 42.0.into()),
            ("0042", 42.0.into()),
            ("1.25e2", 125.0.into()),
            ("1.25e-2", 0.0125.into()),
            ("  42", 42.0.into()),
            ("\t42", 42.0.into()),
            ("42  ", 42.0.into()),
            ("  42  ", 42.0.into()),
            ("true", true.into()),
            ("false", false.into()),
            ("  false  ", false.into()),
            ("#FFAA00", Color::from_hex("#FFAA00").into()),
            ("#FFAA00CC", Color::from_hex("#FFAA00CC").into()),
            ("#ffaa00", Color::from_hex("#FFAA00").into()),
            ("#fFaA00", Color::from_hex("#FFAA00").into()),
            ("  #fFaA00  ", Color::from_hex("#FFAA00").into()),
            (r#""text""#, "text".to_string().into()),
            (r#""Текст!ёё""#, "Текст!ёё".to_string().into()),
            (r#""택스트""#, "택스트".to_string().into()),
            (
                r#""Escaped \" quotes""#,
                r#"Escaped " quotes"#.to_string().into(),
            ),
            (r#"'text'"#, "text".to_string().into()),
            (
                r#"'Escaped \' quotes'"#,
                "Escaped ' quotes".to_string().into(),
            ),
            (r#"  "text"  "#, "text".to_string().into()),
            ("property", Expr::Get("property".to_string())),
            ("nullish", Expr::Get("nullish".to_string())),
            ("trueman", Expr::Get("trueman".to_string())),
            ("  property  ", Expr::Get("property".to_string())),
            ("zoom()", Expr::Zoom),
            ("  zoom()  ", Expr::Zoom),
            ("get('property')", Expr::Get("property".to_string())),
            (
                "any([false, bool_prop])",
                Expr::Any(vec![false.into(), Expr::Get("bool_prop".to_owned())]),
            ),
            (
                "all([string_prop == 'hi', bool_prop])",
                Expr::All(vec![
                    Expr::Eq(
                        Box::new(Expr::Get("string_prop".to_owned())),
                        Box::new("hi".to_owned().into()),
                    ),
                    Expr::Get("bool_prop".to_owned()),
                ]),
            ),
            ("not(true)", Expr::Not(Box::new(true.into()))),
            (
                "not(not(bool_prop))",
                Expr::Not(Box::new(Expr::Not(Box::new(Expr::Get(
                    "bool_prop".to_string(),
                ))))),
            ),
            (
                "in(param, ['candidate1', 'candidate2'])",
                Expr::In {
                    needle: Box::new(Expr::Get("param".to_owned())),
                    haystack: vec![
                        "candidate1".to_owned().into(),
                        "candidate2".to_owned().into(),
                    ],
                },
            ),
            ("geom_type()", Expr::GeomType),
        ];

        for (case, expected) in cases {
            check(case, expected);
        }
    }

    #[test]
    fn parse_binary() {
        check(
            "2 == 3",
            Expr::Eq(Box::new(2.0.into()), Box::new(3.0.into())),
        );
        check(
            "trueman == 42",
            Expr::Eq(
                Box::new(Expr::Get("trueman".to_string())),
                Box::new(42.0.into()),
            ),
        );
    }

    #[test]
    fn parse_block() {
        check("(42)", Expr::Literal(42.0.into()));
        check(
            "(42 == 12)",
            Expr::Eq(Box::new(42.0.into()), Box::new(12.0.into())),
        );
        check(
            "(42 == 12) == false",
            Expr::Eq(
                Box::new(Expr::Eq(Box::new(42.0.into()), Box::new(12.0.into()))),
                Box::new(false.into()),
            ),
        );
        check(
            "true == (42 == 12)",
            Expr::Eq(
                Box::new(true.into()),
                Box::new(Expr::Eq(Box::new(42.0.into()), Box::new(12.0.into()))),
            ),
        );
    }

    fn s(input: &str) -> String {
        let result = expr_parser().parse(input).into_result();
        assert!(
            result.is_err(),
            "Expression `{input}` should be parsed with error, but returned: {result:#?}"
        );

        format!("{:?}", result.err().unwrap()[0])
    }

    use insta::assert_debug_snapshot as ass; // Assert SnapShot

    #[test]
    fn parser_error_num() {
        ass!(s("1.2.3"), @r#""invalid value `1.2.3` at 0..5""#);
    }

    #[test]
    fn parser_error_literal() {
        ass!(s("???"), @r#""invalid value `???` at 0..3""#)
    }

    #[test]
    fn parser_error_unary_with_literal() {
        ass!(s("property == ???"), @r#""invalid value `???` at 12..15""#)
    }

    #[test]
    fn parser_error_two_literals() {
        ass!(s("property 'value'"), @r#""found ''\\''' at 9..10 expected ==, !=, >, >=, <, <=, or end of input""#)
    }

    #[test]
    fn parser_error_unknown_function() {
        ass!(s("unknown_function()"), @r#""Unknown function `unknown_function` at 0..16""#);
    }

    #[test]
    fn parser_error_invalid_argument_count() {
        ass!(s("zoom(property)"), @r#""function zoom expected 0 arguments but got 1 instead at 0..14""#);
    }

    #[test]
    fn parser_error_invalid_argument_type_string() {
        ass!(s("get(42)"), @r#""expected `String` argument at position `1` of function `get`, but got `42` at 4..6""#);
    }

    #[test]
    fn parser_error_invalid_argument_type_array() {
        ass!(s("any(true)"), @r#""expected `[Array]` argument at position `1` of function `any`, but got `true` at 4..8""#);
    }

    #[test]
    fn parser_error_unclosed_bracket() {
        ass!(s("zoom("), @r#""found end of input at 5..5 expected ''['', ''('', expression block, literal, any, property name, or '')''""#);
    }

    #[test]
    fn parser_error_unclosed_array() {
        ass!(s("any([true, false)"), @r#""found '')'' at 16..17 expected ==, !=, >, >=, <, <=, '','', or '']''""#);
    }
}
