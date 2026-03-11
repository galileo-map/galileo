use galileo::Color;

/// Parses a CSS color string into a [`Color`].
///
/// Delegates to the `csscolorparser` crate, so all CSS color formats are supported.
/// Returns `None` for invalid input.
pub fn parse_css_color(s: &str) -> Option<Color> {
    let c = csscolorparser::parse(s).ok()?;
    Some(Color::rgba(
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_short() {
        let c = parse_css_color("#f00").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 0, 0));
    }

    #[test]
    fn hex_long() {
        let c = parse_css_color("#ff8000").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 128, 0));
    }

    #[test]
    fn rgb() {
        let c = parse_css_color("rgb(10, 20, 30)").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (10, 20, 30));
    }

    #[test]
    fn rgba() {
        let c = parse_css_color("rgba(10, 20, 30, 0.5)").unwrap();
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (10, 20, 30, 128));
    }

    #[test]
    fn hsl_red() {
        let c = parse_css_color("hsl(0, 100%, 50%)").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 0, 0));
    }

    #[test]
    fn hsla() {
        let c = parse_css_color("hsla(0, 100%, 50%, 0.5)").unwrap();
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (255, 0, 0, 128));
    }
}
