use crate::dimensions::Unit;
use crate::font::{Weight, Family, Symbol, AtomType};
use crate::font::Style;
use crate::font::style::style_symbol;
use crate::layout::Style as LayoutStyle;
use crate::lexer::{Lexer, Token};
use crate::parser as parse;
use crate::parser::nodes::{ParseNode, Radical, MathStyle, GenFraction, Rule, BarThickness, AtomChange,
                    Color, Stack};
use crate::parser::color::RGBA;
use crate::error::{Error, Result};



macro_rules! sym {
    (@at ord) => { AtomType::Ordinal };
    (@at bin) => { AtomType::Binary };
    (@at op)  => { AtomType::Operator };
    (@at open) => { AtomType::Open };
    (@at close) => { AtomType::Close };

    ($code:expr, $ord:ident) => ({
        Some(Symbol {
            unicode: $code as u32,
            atom_type: sym!(@at $ord),
        })
    });
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command {
    Radical,
    Rule,
    VExtend,
    Color,
    ColorLit(RGBA),
    Fraction(Option<Symbol>, Option<Symbol>, BarThickness, MathStyle),
    DelimiterSize(u8, AtomType),
    Kerning(Unit),
    Style(LayoutStyle),
    AtomChange(AtomType),
    TextOperator(&'static str, bool),
    SubStack(AtomType),
    OperatorName,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Command {
    pub fn parse(self, lex: &mut Lexer, local: Style) -> Result<ParseNode> {
        use self::Command::*;
        match self {
            Radical              => radical(lex, local),
            Rule                 => rule(lex, local),
            VExtend              => v_extend(lex, local),
            Color                => color(lex, local),
            ColorLit(a)          => color_lit(lex, local, a),
            Fraction(a, b, c, d) => fraction(lex, local, a, b, c, d),
            DelimiterSize(a, b)  => delimiter_size(lex, local, a, b),
            Kerning(a)           => kerning(lex, local, a),
            Style(a)             => style(lex, local, a),
            AtomChange(a)        => atom_change(lex, local, a),
            TextOperator(a, b)   => text_operator(lex, local, a, b),
            SubStack(a)          => substack(lex, local, a),
            OperatorName         => operatorname(lex, local),
        }
    }
}


pub static COMMANDS: std::sync::LazyLock<std::collections::HashMap<&'static str, Command>> = std::sync::LazyLock::new(|| {
    let mut m = std::collections::HashMap::new();
    m.insert("frac", Command::Fraction(None, None, BarThickness::Default, MathStyle::NoChange));
    m.insert("tfrac", Command::Fraction(None, None, BarThickness::Default, MathStyle::Text));
    m.insert("dfrac", Command::Fraction(None, None, BarThickness::Default, MathStyle::Display));
    m.insert("binom", Command::Fraction(sym!(b'(', open), sym!(b')', close), BarThickness::None, MathStyle::NoChange));
    m.insert("tbinom", Command::Fraction(sym!(b'(', open), sym!(b')', close), BarThickness::None, MathStyle::Text));
    m.insert("dbinom", Command::Fraction(sym!(b'(', open), sym!(b')', close), BarThickness::None, MathStyle::Display));
    m.insert("substack", Command::SubStack(AtomType::Inner));
    m.insert("sqrt", Command::Radical);
    m.insert("bigl", Command::DelimiterSize(1, AtomType::Open));
    m.insert("Bigl", Command::DelimiterSize(2, AtomType::Open));
    m.insert("biggl", Command::DelimiterSize(3, AtomType::Open));
    m.insert("Biggl", Command::DelimiterSize(4, AtomType::Open));
    m.insert("bigr", Command::DelimiterSize(1, AtomType::Close));
    m.insert("Bigr", Command::DelimiterSize(2, AtomType::Close));
    m.insert("biggr", Command::DelimiterSize(3, AtomType::Close));
    m.insert("Biggr", Command::DelimiterSize(4, AtomType::Close));
    m.insert("bigm", Command::DelimiterSize(1, AtomType::Relation));
    m.insert("Bigm", Command::DelimiterSize(2, AtomType::Relation));
    m.insert("biggm", Command::DelimiterSize(3, AtomType::Relation));
    m.insert("Biggm", Command::DelimiterSize(4, AtomType::Relation));
    m.insert("big", Command::DelimiterSize(1, AtomType::Ordinal));
    m.insert("Big", Command::DelimiterSize(2, AtomType::Ordinal));
    m.insert("bigg", Command::DelimiterSize(3, AtomType::Ordinal));
    m.insert("Bigg", Command::DelimiterSize(4, AtomType::Ordinal));
    m.insert("!", Command::Kerning(Unit::Em(-3f64/18f64)));
    m.insert(",", Command::Kerning(Unit::Em(3f64/18f64)));
    m.insert(":", Command::Kerning(Unit::Em(4f64/18f64)));
    m.insert(";", Command::Kerning(Unit::Em(5f64/18f64)));
    m.insert(" ", Command::Kerning(Unit::Em(1f64/4f64)));
    m.insert("quad", Command::Kerning(Unit::Em(1.0f64)));
    m.insert("qquad", Command::Kerning(Unit::Em(2.0f64)));
    m.insert("rule", Command::Rule);
    m.insert("vextend", Command::VExtend);
    m.insert("textstyle", Command::Style(LayoutStyle::Text));
    m.insert("displaystyle", Command::Style(LayoutStyle::Display));
    m.insert("scriptstyle", Command::Style(LayoutStyle::Script));
    m.insert("scriptscriptstyle", Command::Style(LayoutStyle::ScriptScript));
    m.insert("mathop", Command::AtomChange(AtomType::Operator(false)));
    m.insert("mathrel", Command::AtomChange(AtomType::Relation));
    m.insert("mathord", Command::AtomChange(AtomType::Alpha));
    m.insert("color", Command::Color);
    m.insert("blue", Command::ColorLit(RGBA(0,0,0xff,0xff)));
    m.insert("red", Command::ColorLit(RGBA(0xff,0,0,0xff)));
    m.insert("gray", Command::ColorLit(RGBA(0x80,0x80,0x80,0xff)));
    m.insert("phantom", Command::ColorLit(RGBA(0,0,0,0)));
    m.insert("det", Command::TextOperator("det", true));
    m.insert("gcd", Command::TextOperator("gcd", true));
    m.insert("lim", Command::TextOperator("lim", true));
    m.insert("limsup", Command::TextOperator("lim,sup", true));
    m.insert("liminf", Command::TextOperator("lim,inf", true));
    m.insert("sup", Command::TextOperator("sup", true));
    m.insert("supp", Command::TextOperator("supp", true));
    m.insert("inf", Command::TextOperator("inf", true));
    m.insert("max", Command::TextOperator("max", true));
    m.insert("min", Command::TextOperator("min", true));
    m.insert("Pr", Command::TextOperator("Pr", true));
    m.insert("sin", Command::TextOperator("sin", false));
    m.insert("cos", Command::TextOperator("cos", false));
    m.insert("tan", Command::TextOperator("tan", false));
    m.insert("cot", Command::TextOperator("cot", false));
    m.insert("csc", Command::TextOperator("csc", false));
    m.insert("sec", Command::TextOperator("sec", false));
    m.insert("arcsin", Command::TextOperator("arcsin", false));
    m.insert("arccos", Command::TextOperator("arccos", false));
    m.insert("arctan", Command::TextOperator("arctan", false));
    m.insert("sinh", Command::TextOperator("sinh", false));
    m.insert("cosh", Command::TextOperator("cosh", false));
    m.insert("tanh", Command::TextOperator("tanh", false));
    m.insert("arg", Command::TextOperator("arg", false));
    m.insert("deg", Command::TextOperator("deg", false));
    m.insert("dim", Command::TextOperator("dim", false));
    m.insert("exp", Command::TextOperator("exp", false));
    m.insert("hom", Command::TextOperator("hom", false));
    m.insert("Hom", Command::TextOperator("Hom", false));
    m.insert("ker", Command::TextOperator("ker", false));
    m.insert("Ker", Command::TextOperator("Ker", false));
    m.insert("ln", Command::TextOperator("ln", false));
    m.insert("log", Command::TextOperator("log", false));

    // \operatorname{...} - dynamic operator name
    m.insert("operatorname", Command::OperatorName);
    m.insert("operatorname*", Command::OperatorName);

    // Additional common operators LLMs like to emit
    m.insert("sgn", Command::TextOperator("sgn", false));
    m.insert("Tr", Command::TextOperator("Tr", false));
    m.insert("tr", Command::TextOperator("tr", false));
    m.insert("rank", Command::TextOperator("rank", false));
    m.insert("Rank", Command::TextOperator("Rank", false));
    m.insert("diag", Command::TextOperator("diag", false));
    m.insert("lcm", Command::TextOperator("lcm", true));
    m.insert("Sel", Command::TextOperator("Sel", false));
    m.insert("Gal", Command::TextOperator("Gal", false));
    m.insert("Aut", Command::TextOperator("Aut", false));
    m.insert("End", Command::TextOperator("End", false));
    m.insert("Ext", Command::TextOperator("Ext", false));
    m.insert("Tor", Command::TextOperator("Tor", false));
    m.insert("GL", Command::TextOperator("GL", false));
    m.insert("SL", Command::TextOperator("SL", false));
    m.insert("SO", Command::TextOperator("SO", false));
    m.insert("SU", Command::TextOperator("SU", false));
    m.insert("Spec", Command::TextOperator("Spec", false));
    m.insert("Proj", Command::TextOperator("Proj", false));
    m.insert("Hess", Command::TextOperator("Hess", false));
    m.insert("Res", Command::TextOperator("Res", false));
    m.insert("Re", Command::TextOperator("Re", false));
    m.insert("Im", Command::TextOperator("Im", false));
    m.insert("ord", Command::TextOperator("ord", false));
    m.insert("mod", Command::TextOperator("mod", false));
    m.insert("Var", Command::TextOperator("Var", false));
    m.insert("Cov", Command::TextOperator("Cov", false));
    m.insert("Corr", Command::TextOperator("Corr", false));
    m.insert("argmax", Command::TextOperator("arg,max", true));
    m.insert("argmin", Command::TextOperator("arg,min", true));
    m.insert("softmax", Command::TextOperator("softmax", false));
    m.insert("Span", Command::TextOperator("Span", false));
    m.insert("codim", Command::TextOperator("codim", false));
    m.insert("coker", Command::TextOperator("coker", false));
    m.insert("Char", Command::TextOperator("Char", false));
    m
});


fn radical(lex: &mut Lexer, local: Style) -> Result<ParseNode> {
    let inner = parse::required_argument(lex, local)?;
    Ok(ParseNode::Radical(Radical { inner }))
}

fn rule(lex: &mut Lexer, _: Style) -> Result<ParseNode> {
    lex.consume_whitespace();
    let width = lex.dimension()?
        .expect("Unable to parse dimension for Rule.");
    lex.consume_whitespace();
    let height = lex.dimension()?
        .expect("Unable to parse dimension for Rule.");
    Ok(ParseNode::Rule(Rule { width, height }))
}

fn v_extend(lex: &mut Lexer, local: Style) -> Result<ParseNode> {
    let arg = parse::required_argument_with(lex, local, parse::symbol)?;
    let sym = match arg {
        Some(ParseNode::Symbol(sym)) => sym,

        // TODO: add better error
        _ => return Err(Error::ExpectedOpenGroup),
    };

    let height = parse::required_argument_with(lex, local, parse::dimension)?;
    Ok(ParseNode::Extend(sym.unicode, height))
}

fn color(lex: &mut Lexer, local: Style) -> Result<ParseNode> {
    let color = parse::required_argument_with(lex, local, parse::color)?;
    let inner = parse::required_argument(lex, local)?;
    Ok(ParseNode::Color(Color { color, inner }))
}

fn color_lit(lex: &mut Lexer, local: Style, color: RGBA) -> Result<ParseNode> {
    let inner = parse::required_argument(lex, local)?;
    Ok(ParseNode::Color(Color { color, inner }))
}

fn fraction(lex: &mut Lexer,
            local: Style,
            left_delimiter: Option<Symbol>,
            right_delimiter: Option<Symbol>,
            bar_thickness: BarThickness,
            style: MathStyle)
            -> Result<ParseNode> {
    let numerator = parse::required_argument(lex, local)?;
    let denominator = parse::required_argument(lex, local)?;

    Ok(ParseNode::GenFraction(GenFraction {
                                  left_delimiter,
                                  right_delimiter,
                                  bar_thickness,
                                  numerator,
                                  denominator,
                                  style,
                              }))
}

fn delimiter_size(lex: &mut Lexer, local: Style, _: u8, atom_type: AtomType) -> Result<ParseNode> {
    let symbol = parse::expect_type(lex, local, atom_type)?;
    Ok(ParseNode::Symbol(symbol))
}

fn kerning(_: &mut Lexer, _: Style, unit: Unit) -> Result<ParseNode> {
    Ok(ParseNode::Kerning(unit))
}

fn style(_: &mut Lexer, _: Style, new_style: LayoutStyle) -> Result<ParseNode> {
    Ok(ParseNode::Style(new_style))
}

fn atom_change(lex: &mut Lexer, local: Style, at: AtomType) -> Result<ParseNode> {
    let inner = parse::required_argument(lex, local)?;
    Ok(ParseNode::AtomChange(AtomChange { at, inner }))
}

fn text_operator(_: &mut Lexer, _: Style, text: &str, limits: bool) -> Result<ParseNode> {
    const SMALL_SKIP: Unit = Unit::Em(3f64 / 18f64);
    let at = AtomType::Operator(limits);
    let mut inner = Vec::with_capacity(text.len());

    for c in text.chars() {
        if c == ',' {
            inner.push(ParseNode::Kerning(SMALL_SKIP));
        } else {
            inner.push(ParseNode::Symbol(Symbol {
                                             unicode:
                                                 style_symbol(c as u32,
                                                              Style::default()
                                                                  .with_family(Family::Roman)
                                                                  .with_weight(Weight::None)),
                                             atom_type: AtomType::Ordinal,
                                         }));
        }
    }

    Ok(ParseNode::AtomChange(AtomChange { at, inner }))
}

fn substack(lex: &mut Lexer, local: Style, atom_type: AtomType) -> Result<ParseNode> {
    if lex.current != Token::Symbol('{') {
        return Err(Error::StackMustFollowGroup);
    }

    let mut lines: Vec<Vec<ParseNode>> = Vec::new();
    lex.next();

    // Continue parsing expressions, until we reach '}'
    loop {
        lines.push(parse::expression(lex, local)?);
        match lex.current {
            Token::Symbol('}') => break,
            Token::Command(r"\") => lex.next(),
            _ => return Err(Error::Todo),
        };
    }

    // Remove the last line if it's empty.  This is `\crcr`.
    if Some(true) == lines.last().map(|l| l.is_empty()) {
        lines.pop();
    }

    lex.next();
    Ok(ParseNode::Stack(Stack { atom_type, lines }))
}

/// `\operatorname{Name}` - renders `Name` as an upright operator with proper spacing.
/// This reads the braced argument as individual characters and renders them in Roman style,
/// just like `\sin`, `\cos`, etc. but with a dynamic name.
fn operatorname(lex: &mut Lexer, _local: Style) -> Result<ParseNode> {
    // Expect a `{`
    if lex.current != Token::Symbol('{') {
        return Err(Error::ExpectedOpenGroup);
    }
    lex.next();

    // Read characters until `}`
    let mut name = String::new();
    loop {
        match lex.current {
            Token::Symbol('}') => {
                lex.next();
                break;
            }
            Token::Symbol(c) => {
                name.push(c);
                lex.next();
            }
            Token::Command(cmd) => {
                // Some people write \operatorname{\mathcal{F}} etc. - just grab the text
                name.push_str(cmd);
                lex.next();
            }
            Token::EOF => return Err(Error::NoClosingBracket),
            _ => {
                lex.next();
            }
        }
    }

    let at = AtomType::Operator(false);
    let mut inner = Vec::with_capacity(name.len());
    for c in name.chars() {
        inner.push(ParseNode::Symbol(Symbol {
            unicode: style_symbol(
                c as u32,
                Style::default()
                    .with_family(Family::Roman)
                    .with_weight(Weight::None),
            ),
            atom_type: AtomType::Ordinal,
        }));
    }

    Ok(ParseNode::AtomChange(AtomChange { at, inner }))
}
