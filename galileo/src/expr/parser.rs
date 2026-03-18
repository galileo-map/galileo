use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while, take_while_m_n, take_while1};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, not, peek, value};
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};

use super::{Expr, ExprValue};
use crate::Color;

type ParseResult<'a, T> = IResult<&'a str, T>;

/// Parses a complete expression from a string slice.
pub fn parse_expr(input: &str) -> ParseResult<'_, Expr> {
    ws(expr)(input)
}

fn ws<'a, F, O>(f: F) -> impl FnMut(&'a str) -> ParseResult<'a, O>
where
    F: FnMut(&'a str) -> ParseResult<'a, O>,
{
    delimited(multispace0, f, multispace0)
}

fn expr(input: &str) -> ParseResult<'_, Expr> {
    alt((comparison_expr, logical_expr, not_expr, in_expr, atom))(input)
}

/// Parses an atom: a literal or an identifier (which is either a known function call or a `Get`).
fn atom(input: &str) -> ParseResult<'_, Expr> {
    alt((map(literal, Expr::Literal), ident_or_call))(input)
}

/// Parses an identifier. An identifier starts with a letter or `_` and may contain letters,
/// digits, `_`, or `-`.
fn ident(input: &str) -> ParseResult<'_, &str> {
    let (input, first) = take_while1(|c: char| c.is_ascii_alphabetic() || c == '_')(input)?;
    let (input, rest) =
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')(input)?;
    // Combine by returning the slice that spans both parts.
    let full = &input[..0]; // zero-length suffix; we compute the span manually
    let _ = full;
    let span_len = first.len() + rest.len();
    Ok((input, &first[..span_len]))
}

/// Dispatches an identifier to a known zero-arg function (`geom_type`, `zoom`), a known
/// function call with arguments (`all`, `any`, `not`, `in`, `rgba`, `hsv`), or a bare
/// identifier treated as a `Get` expression.
fn ident_or_call(input: &str) -> ParseResult<'_, Expr> {
    let (input, name) = ident(input)?;
    // Peek: if the next non-space char is `(`, this is a function call.
    let after_ws = input.trim_start();
    if after_ws.starts_with('(') {
        function_call(name, input)
    } else {
        Ok((input, Expr::Get(name.to_string())))
    }
}

/// Parses the argument list `( args… )` for a named function and builds the corresponding `Expr`.
fn function_call<'a>(name: &str, input: &'a str) -> ParseResult<'a, Expr> {
    match name {
        "geom_type" => {
            let (input, _) = delimited(ws(char('(')), multispace0, ws(char(')')))(input)?;
            Ok((input, Expr::GeomType))
        }
        "zoom" => {
            let (input, _) = delimited(ws(char('(')), multispace0, ws(char(')')))(input)?;
            Ok((input, Expr::Zoom))
        }
        "not" => {
            let (input, inner) = delimited(ws(char('(')), ws(expr), ws(char(')')))(input)?;
            Ok((input, Expr::Not(Box::new(inner))))
        }
        "all" => {
            let (input, exprs) = delimited(ws(char('(')), ws(expr_list), ws(char(')')))(input)?;
            Ok((input, Expr::All(exprs)))
        }
        "any" => {
            let (input, exprs) = delimited(ws(char('(')), ws(expr_list), ws(char(')')))(input)?;
            Ok((input, Expr::Any(exprs)))
        }
        "in" => {
            let (input, (needle, haystack)) = delimited(
                ws(char('(')),
                separated_pair(ws(expr), ws(char(',')), ws(expr_list)),
                ws(char(')')),
            )(input)?;
            Ok((
                input,
                Expr::In {
                    needle: Box::new(needle),
                    haystack,
                },
            ))
        }
        "rgba" => {
            let (input, (r, g, b, a)) = delimited(
                ws(char('(')),
                tuple((
                    ws(double_u8),
                    preceded(ws(char(',')), ws(double_u8)),
                    preceded(ws(char(',')), ws(double_u8)),
                    preceded(ws(char(',')), ws(double_u8)),
                )),
                ws(char(')')),
            )(input)?;
            Ok((
                input,
                Expr::Literal(ExprValue::Color(Color::rgba(r, g, b, a))),
            ))
        }
        "hsv" => {
            let (input, (h, s, v)) = delimited(
                ws(char('(')),
                tuple((
                    ws(double),
                    preceded(ws(char(',')), ws(double)),
                    preceded(ws(char(',')), ws(double)),
                )),
                ws(char(')')),
            )(input)?;
            Ok((
                input,
                Expr::Literal(ExprValue::Color(color_from_hsv(h, s, v))),
            ))
        }
        // Unknown function name: treat the bare identifier as a Get and leave `(` unparsed.
        _ => Ok((input, Expr::Get(name.to_string()))),
    }
}

/// Parses a quoted string literal. Both `"hello"` and `'hello'` are accepted; the opening and
/// closing quote must match.
fn string_literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    let (input, quote) = alt((char('"'), char('\'')))(input)?;
    let (input, s) = take_while(move |c| c != quote)(input)?;
    let (input, _) = char(quote)(input)?;
    Ok((input, ExprValue::String(s.to_string())))
}

/// Parses a numeric literal, but rejects bare identifiers that `double` would otherwise consume.
fn number_literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    // Reject if the input starts with a letter (would be an ident / keyword).
    let (_, _) = not(peek(take_while1(|c: char| {
        c.is_ascii_alphabetic() || c == '_'
    })))(input)?;
    map(double, ExprValue::Number)(input)
}

/// Parses `true` or `false` as boolean literals, ensuring they are not a prefix of a longer ident.
fn bool_literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    alt((
        map(terminated(tag("true"), not(peek(ident_cont))), |_| {
            ExprValue::Boolean(true)
        }),
        map(terminated(tag("false"), not(peek(ident_cont))), |_| {
            ExprValue::Boolean(false)
        }),
    ))(input)
}

/// Parses `null`, ensuring it is not a prefix of a longer identifier.
fn null_literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    value(
        ExprValue::Null,
        terminated(tag("null"), not(peek(ident_cont))),
    )(input)
}

/// Matches a single identifier-continuation character.
fn ident_cont(input: &str) -> ParseResult<'_, char> {
    nom::character::complete::satisfy(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')(
        input,
    )
}

/// Parses a hex color literal: `#RRGGBB` or `#RRGGBBAA`.
fn hex_color_literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    let (input, _) = char('#')(input)?;
    let (input, hex) = take_while_m_n(6, 8, |c: char| c.is_ascii_hexdigit())(input)?;
    if hex.len() != 6 && hex.len() != 8 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }
    let color_str = format!("#{hex}");
    match Color::try_from_hex(&color_str) {
        Some(c) => Ok((input, ExprValue::Color(c))),
        None => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Fail,
        ))),
    }
}

fn literal(input: &str) -> ParseResult<'_, ExprValue<String>> {
    alt((
        bool_literal,
        null_literal,
        hex_color_literal,
        number_literal,
        string_literal,
    ))(input)
}

/// Parses a `double` value and clamps it to a `u8` (0–255).
fn double_u8(input: &str) -> ParseResult<'_, u8> {
    map(double, |v: f64| v.clamp(0.0, 255.0) as u8)(input)
}

/// Converts HSV (hue 0–360, saturation 0–1, value 0–1) to `Color`.
fn color_from_hsv(h: f64, s: f64, v: f64) -> Color {
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::rgba(
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
        255,
    )
}

/// Parses binary comparison operators: `==`, `!=`, `>=`, `>`, `<=`, `<`.
fn comparison_expr(input: &str) -> ParseResult<'_, Expr> {
    let (input, (lhs, op, rhs)) = tuple((
        ws(atom),
        ws(alt((
            tag("=="),
            tag("!="),
            tag(">="),
            tag(">"),
            tag("<="),
            tag("<"),
        ))),
        ws(atom),
    ))(input)?;

    let lhs = Box::new(lhs);
    let rhs = Box::new(rhs);
    let result = match op {
        "==" => Expr::Eq(lhs, rhs),
        "!=" => Expr::Ne(lhs, rhs),
        ">" => Expr::Gt(lhs, rhs),
        ">=" => Expr::Gte(lhs, rhs),
        "<" => Expr::Lt(lhs, rhs),
        "<=" => Expr::Lte(lhs, rhs),
        _ => unreachable!(),
    };
    Ok((input, result))
}

/// Parses a comma-separated list of expressions wrapped in square brackets.
fn expr_list(input: &str) -> ParseResult<'_, Vec<Expr>> {
    delimited(
        ws(char('[')),
        separated_list0(ws(char(',')), ws(expr)),
        ws(char(']')),
    )(input)
}

/// Parses `!atom`.
fn not_expr(input: &str) -> ParseResult<'_, Expr> {
    map(preceded(ws(char('!')), atom), |e| Expr::Not(Box::new(e)))(input)
}

/// Parses `all([…])` and `any([…])` as shorthands using bracket syntax.
fn logical_expr(input: &str) -> ParseResult<'_, Expr> {
    let (input, op) = alt((
        terminated(tag_no_case("all"), peek(ws(char('[')))),
        terminated(tag_no_case("any"), peek(ws(char('[')))),
    ))(input)?;
    let (input, exprs) = ws(expr_list)(input)?;
    let result = match op.to_ascii_lowercase().as_str() {
        "all" => Expr::All(exprs),
        "any" => Expr::Any(exprs),
        _ => unreachable!(),
    };
    Ok((input, result))
}

/// Parses `in(needle, [a, b, c])`.
fn in_expr(input: &str) -> ParseResult<'_, Expr> {
    let (input, _) = terminated(tag("in"), peek(ws(char('('))))(input)?;
    let (input, (needle, haystack)) = delimited(
        ws(char('(')),
        separated_pair(ws(expr), ws(char(',')), ws(expr_list)),
        ws(char(')')),
    )(input)?;
    Ok((
        input,
        Expr::In {
            needle: Box::new(needle),
            haystack,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_literal_number() {
        let (rest, expr) = parse_expr("42").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Literal(ExprValue::Number(42.0)));
    }

    #[test]
    fn parse_literal_bool() {
        let (rest, expr) = parse_expr("true").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Literal(ExprValue::Boolean(true)));
    }

    #[test]
    fn parse_literal_string() {
        let (rest, expr) = parse_expr(r#""hello""#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Literal(ExprValue::String("hello".to_string())));
    }

    #[test]
    fn parse_literal_string_single_quotes() {
        let (rest, expr) = parse_expr("'hello'").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Literal(ExprValue::String("hello".to_string())));
    }

    #[test]
    fn parse_single_quotes_in_comparison() {
        let (rest, expr) = parse_expr("kind == 'road'").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Eq(
                Box::new(Expr::Get("kind".to_string())),
                Box::new(Expr::Literal(ExprValue::String("road".to_string()))),
            )
        );
    }

    #[test]
    fn parse_get_bare_ident() {
        let (rest, expr) = parse_expr("name").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Get("name".to_string()));
    }

    #[test]
    fn parse_zoom() {
        let (rest, expr) = parse_expr("zoom()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::Zoom);
    }

    #[test]
    fn parse_geom_type() {
        let (rest, expr) = parse_expr("geom_type()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(expr, Expr::GeomType);
    }

    #[test]
    fn parse_eq_bare_idents() {
        let (rest, expr) = parse_expr(r#"kind == "road""#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Eq(
                Box::new(Expr::Get("kind".to_string())),
                Box::new(Expr::Literal(ExprValue::String("road".to_string()))),
            )
        );
    }

    #[test]
    fn parse_not() {
        let (rest, expr) = parse_expr("!true").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Not(Box::new(Expr::Literal(ExprValue::Boolean(true))))
        );
    }

    #[test]
    fn parse_not_fn() {
        let (rest, expr) = parse_expr("not(true)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Not(Box::new(Expr::Literal(ExprValue::Boolean(true))))
        );
    }

    #[test]
    fn parse_in() {
        let (rest, expr) = parse_expr(r#"in(kind, ["road", "path"])"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::In {
                needle: Box::new(Expr::Get("kind".to_string())),
                haystack: vec![
                    Expr::Literal(ExprValue::String("road".to_string())),
                    Expr::Literal(ExprValue::String("path".to_string())),
                ],
            }
        );
    }

    #[test]
    fn parse_all_bracket() {
        let (rest, expr) = parse_expr("all[true, false]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::All(vec![
                Expr::Literal(ExprValue::Boolean(true)),
                Expr::Literal(ExprValue::Boolean(false)),
            ])
        );
    }

    #[test]
    fn parse_all_fn() {
        let (rest, expr) = parse_expr("all([true, false])").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::All(vec![
                Expr::Literal(ExprValue::Boolean(true)),
                Expr::Literal(ExprValue::Boolean(false)),
            ])
        );
    }

    #[test]
    fn parse_color_hex6() {
        let (rest, expr) = parse_expr("#FF8800").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Literal(ExprValue::Color(Color::rgba(0xFF, 0x88, 0x00, 0xFF)))
        );
    }

    #[test]
    fn parse_color_hex8() {
        let (rest, expr) = parse_expr("#FF880080").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Literal(ExprValue::Color(Color::rgba(0xFF, 0x88, 0x00, 0x80)))
        );
    }

    #[test]
    fn parse_color_rgba() {
        let (rest, expr) = parse_expr("rgba(255, 136, 0, 128)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Literal(ExprValue::Color(Color::rgba(255, 136, 0, 128)))
        );
    }

    #[test]
    fn parse_color_hsv() {
        let (rest, expr) = parse_expr("hsv(0, 1, 1)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            expr,
            Expr::Literal(ExprValue::Color(Color::rgba(255, 0, 0, 255)))
        );
    }
}
