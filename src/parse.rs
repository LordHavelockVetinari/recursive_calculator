use malachite::rational_sequences::RationalSequence;
use malachite::{Natural, Rational};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{digit1, satisfy};
use nom::combinator::{map, opt};
use nom::multi::{many0, many0_count, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

#[derive(Debug, thiserror::Error)]
pub enum ParseError<'a> {
    #[error("Parse Error: {0}")]
    Nom(nom::Err<nom::error::Error<&'a str>>),
    #[error("Unexpected character: {:?}", .0.chars().next().unwrap())]
    Incomplete(&'a str),
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for ParseError<'a> {
    fn from(err: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Self::Nom(err)
    }
}

#[derive(Debug)]
pub enum Expression {
    Number(Rational),
    Identifier(String),
    Pos(Box<Expression>),
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}

pub enum Statement {
    Assign(Vec<Expression>),
    Evaluate(Expression),
}

pub struct Code {
    pub statements: Vec<Statement>,
}

fn digits(input: &str) -> IResult<&str, Vec<u8>> {
    map(digit1, |digits: &str| {
        digits
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect()
    })(input)
}

fn number(input: &str) -> IResult<&str, Rational> {
    map(
        pair(digits, opt(preceded(tag("."), digits))),
        |(part1, part2)| {
            let part1 = part1.into_iter().rev().map(Natural::from).collect();
            let part2 = RationalSequence::from_vec(
                part2
                    .unwrap_or_default()
                    .into_iter()
                    .map(Natural::from)
                    .collect(),
            );
            Rational::from_digits(&Natural::from(10u32), part1, part2)
        },
    )(input)
}

fn identifier(input: &str) -> IResult<&str, String> {
    map(
        pair(
            satisfy(|c| c.is_alphabetic() || c == '_'),
            many0(satisfy(|c| c.is_alphanumeric() || c == '_' || c == '\'')),
        ),
        |(c, cs)| {
            let mut s = String::new();
            s.push(c);
            for c in cs {
                s.push(c);
            }
            s
        },
    )(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    let (rest, (_, n)) = pair(tag("**"), many0_count(tag("*")))(input)?;
    let end_comment = "*".repeat(n + 2);
    let mut comb = map(
        pair(take_until(&end_comment[..]), tag(&end_comment[..])),
        |_| (),
    );
    comb(rest)
}

fn ws0(input: &str, newline: bool) -> IResult<&str, ()> {
    if newline {
        map(
            many0(alt((map(satisfy(char::is_whitespace), |_| ()), comment))),
            |_| (),
        )(input)
    } else {
        map(
            many0(alt((
                map(satisfy(|c| c.is_whitespace() && c != '\n'), |_| ()),
                comment,
            ))),
            |_| (),
        )(input)
    }
}

fn parenthesized<'a, F, O>(input: &'a str, f: F) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    let (input, bracket) = alt((tag("("), tag("["), tag("{")))(input)?;
    let (input, result) = preceded(pass_newline(ws0, true), f)(input)?;
    let (input, _) = preceded(
        pass_newline(ws0, true),
        tag(match bracket {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => unreachable!(),
        }),
    )(input)?;
    Ok((input, result))
}

fn pass_newline<F, T>(mut f: F, newline: bool) -> impl FnMut(&str) -> IResult<&str, T>
where
    F: FnMut(&str, bool) -> IResult<&str, T>,
{
    move |input| f(input, newline)
}

fn expr1(input: &str, newline: bool) -> IResult<&str, Expression> {
    alt((
        map(
            pair(
                identifier,
                opt(preceded(pass_newline(ws0, newline), |i| {
                    parenthesized(
                        i,
                        terminated(
                            separated_list1(
                                preceded(pass_newline(ws0, newline), tag(",")),
                                preceded(pass_newline(ws0, true), pass_newline(expr, true)),
                            ),
                            opt(preceded(pass_newline(ws0, true), tag(","))),
                        ),
                    )
                })),
            ),
            |(ident, args)| {
                if let Some(args) = args {
                    Expression::Call(ident, args)
                } else {
                    Expression::Identifier(ident)
                }
            },
        ),
        map(number, Expression::Number),
        |i| parenthesized(i, pass_newline(expr, true)),
    ))(input)
}

fn expr2(input: &str, newline: bool) -> IResult<&str, Expression> {
    map(
        pair(
            pass_newline(expr1, newline),
            opt(preceded(
                preceded(pass_newline(ws0, newline), tag("^")),
                preceded(pass_newline(ws0, newline), pass_newline(expr3, newline)),
            )),
        ),
        |(left, right)| match right {
            Some(right) => Expression::Pow(Box::new(left), Box::new(right)),
            None => left,
        },
    )(input)
}

fn expr3(input: &str, newline: bool) -> IResult<&str, Expression> {
    map(
        pair(
            many0(terminated(
                alt((tag("+"), tag("-"))),
                pass_newline(ws0, newline),
            )),
            pass_newline(expr2, newline),
        ),
        |(ops, mut expr)| {
            for op in ops.into_iter().rev() {
                expr = match op {
                    "+" => Expression::Pos(Box::new(expr)),
                    "-" => Expression::Neg(Box::new(expr)),
                    _ => panic!("unrecognized unary operator"),
                }
            }
            expr
        },
    )(input)
}

fn expr4(input: &str, newline: bool) -> IResult<&str, Expression> {
    map(
        pair(
            pass_newline(expr3, newline),
            many0(pair(
                preceded(pass_newline(ws0, newline), alt((tag("*"), tag("/")))),
                preceded(pass_newline(ws0, newline), pass_newline(expr3, newline)),
            )),
        ),
        |(mut expr, rest)| {
            for (op, right) in rest {
                expr = match op {
                    "*" => Expression::Mul(Box::new(expr), Box::new(right)),
                    "/" => Expression::Div(Box::new(expr), Box::new(right)),
                    _ => panic!("unrecognized binary operator"),
                }
            }
            expr
        },
    )(input)
}

fn expr5(input: &str, newline: bool) -> IResult<&str, Expression> {
    map(
        pair(
            pass_newline(expr4, newline),
            many0(pair(
                preceded(pass_newline(ws0, newline), alt((tag("+"), tag("-")))),
                preceded(pass_newline(ws0, newline), pass_newline(expr4, newline)),
            )),
        ),
        |(mut expr, rest)| {
            for (op, right) in rest {
                expr = match op {
                    "+" => Expression::Add(Box::new(expr), Box::new(right)),
                    "-" => Expression::Sub(Box::new(expr), Box::new(right)),
                    _ => panic!("unrecognized binary operator"),
                }
            }
            expr
        },
    )(input)
}

fn expr(input: &str, newline: bool) -> IResult<&str, Expression> {
    expr5(input, newline)
}

fn statement(input: &str) -> IResult<&str, Statement> {
    map(
        separated_list1(
            preceded(pass_newline(ws0, false), tag("=")),
            preceded(pass_newline(ws0, false), pass_newline(expr, false)),
        ),
        |mut exprs| {
            assert!(!exprs.is_empty());
            if exprs.len() == 1 {
                Statement::Evaluate(exprs.pop().unwrap())
            } else {
                Statement::Assign(exprs)
            }
        },
    )(input)
}

fn program(input: &str) -> IResult<&str, Code> {
    map(
        delimited(
            pass_newline(ws0, true),
            separated_list0(
                delimited(pass_newline(ws0, false), tag("\n"), pass_newline(ws0, true)),
                statement,
            ),
            pass_newline(ws0, true),
        ),
        |statements| Code { statements },
    )(input)
}

pub fn parse(input: &str) -> Result<Code, ParseError<'_>> {
    let (rest, prog) = program(input)?;
    if !rest.is_empty() {
        return Err(ParseError::Incomplete(rest));
    }
    Ok(prog)
}

#[cfg(test)]
mod test {
    use super::*;

    fn frac(numer: u64, denom: u64) -> Rational {
        Rational::from(numer) / Rational::from(denom)
    }

    #[test]
    fn number_test() {
        let (rest, n) = number("123 + 456").unwrap();
        assert_eq!(n, frac(123, 1));
        assert_eq!(rest, " + 456");

        let (rest, n) = number("123.45").unwrap();
        assert_eq!(n, frac(12345, 100));
        assert_eq!(rest, "");

        assert!(number(".123").is_err());
    }

    #[test]
    fn identifier_test() {
        let (rest, ident) = identifier("Hello, World").unwrap();
        assert_eq!(ident, "Hello");
        assert_eq!(rest, ", World");

        let (rest, ident) = identifier("_x'#").unwrap();
        assert_eq!(ident, "_x'");
        assert_eq!(rest, "#");

        assert!(identifier("'x").is_err());
    }

    #[test]
    fn comment_test() {
        let (rest, ()) = comment("** Hello ***a").unwrap();
        assert_eq!(rest, "*a");

        let (rest, ()) = comment("*** ABC ** DEF * G****HIJ").unwrap();
        assert_eq!(rest, "*HIJ");

        let (rest, ()) = comment(
            "\
            ***********\n\
            * Fancy   *\n\
            * Comment *\n\
            ***********\n\
            Rest",
        )
        .unwrap();
        assert_eq!(rest, "\nRest");

        assert!(comment("* Hello *").is_err());
    }

    #[test]
    fn ws0_test() {
        let (rest, ()) = ws0("  \t\nx", false).unwrap();
        assert_eq!(rest, "\nx");

        let (rest, ()) = ws0("\t\n\rxyz", true).unwrap();
        assert_eq!(rest, "xyz");

        let (rest, ()) = ws0("1+1", true).unwrap();
        assert_eq!(rest, "1+1");
    }

    #[test]
    fn simple_expression_test() {
        let (rest, expr) = expr1("xyz+", false).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "xyz"));
        assert_eq!(rest, "+");

        let (rest, expr) = expr1("A'**x**B", true).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "A'"));
        assert_eq!(rest, "**x**B");

        let (rest, expr) = expr1("x^y", false).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "x"));
        assert_eq!(rest, "^y");

        assert!(expr1("'A", true).is_err());

        let (rest, expr) = expr1("123.456.3", false).unwrap();
        assert!(matches!(expr, Expression::Number(n) if n == frac(123456, 1000)));
        assert_eq!(rest, ".3");

        let (rest, expr) = expr1("123x4", true).unwrap();
        assert!(matches!(expr, Expression::Number(n) if n == frac(123, 1)));
        assert_eq!(rest, "x4");

        assert!(expr1("'x", false).is_err());
    }

    #[test]
    fn parenthesis_test() {
        let (rest, expr) = expr1("(1)x", false).unwrap();
        assert!(matches!(expr, Expression::Number(n) if n == frac(1, 1)));
        assert_eq!(rest, "x");

        let (rest, expr) = expr1("(\n 1 \n)x", false).unwrap();
        assert!(matches!(expr, Expression::Number(n) if n == frac(1, 1)));
        assert_eq!(rest, "x");

        assert!(expr1("(1", true).is_err());
        assert!(expr1("(1, 2)", true).is_err());
    }

    #[test]
    fn call_test() {
        let (rest, expr) = expr1("f ( x , y \n)", false).unwrap();
        assert!(matches!(
            expr,
            Expression::Call(func, args)
            if func == "f"
            && matches!(
                &args[..],
                [Expression::Identifier(x), Expression::Identifier(y)]
                if x == "x" && y == "y",
            ),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr1("foo ( \n1)", false).unwrap();
        assert!(matches!(
            expr,
            Expression::Call(func, args)
            if func == "foo"
            && matches!(
                &args[..],
                [Expression::Number(n)] if *n == frac(1, 1),
            ),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr1("foo()", true).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "foo"));
        assert_eq!(rest, "()");

        let (rest, expr) = expr1("foo[bar,]", false).unwrap();
        assert!(matches!(
            expr,
            Expression::Call(foo, bar)
            if foo == "foo"
            && matches!(&bar[..], [Expression::Identifier(s)] if s == "bar"),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr1("foo{bar, baz}", false).unwrap();
        assert!(matches!(
            expr,
            Expression::Call(foo, args)
            if foo == "foo"
            && matches!(
                &args[..],
                [Expression::Identifier(bar), Expression::Identifier(baz)]
                if bar == "bar" && baz == "baz",
            ),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr1("f{x, y)", false).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "f"));
        assert_eq!(rest, "{x, y)");

        let (rest, expr) = expr1("f(x, y]", false).unwrap();
        assert!(matches!(expr, Expression::Identifier(s) if s == "f"));
        assert_eq!(rest, "(x, y]");
    }

    #[test]
    fn pow_unary_expression_test() {
        let (rest, expr) = expr2("1^2x", false).unwrap();
        assert!(matches!(expr, Expression::Pow(left, right)
            if matches!(&*left, Expression::Number(n) if *n == frac(1, 1))
            && matches!(&*right, Expression::Number(m) if *m == frac(2, 1)),
        ));
        assert_eq!(rest, "x");

        let (rest, expr) = expr2("x\r\n^\n\r y z", true).unwrap();
        assert!(matches!(expr, Expression::Pow(left, right)
            if matches!(&*left, Expression::Identifier(x) if x == "x")
            && matches!(&*right, Expression::Identifier(y) if y == "y"),
        ));
        assert_eq!(rest, " z");

        let (rest, expr) = expr3("-+-3", false).unwrap();
        assert!(matches!(
            expr,
            Expression::Neg(pos) if matches!(
                &*pos,
                Expression::Pos(neg) if matches!(
                    &**neg,
                    Expression::Neg(num) if matches!(
                        &**num,
                        Expression::Number(n) if *n == frac(3, 1),
                    ),
                ),
            ),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr3("- x \n^  + \ty^ -z", true).unwrap();
        assert!(matches!(
            expr,
            Expression::Neg(pow1) if matches!(
                &*pow1,
                Expression::Pow(x, pos)
                if matches!(&**x, Expression::Identifier(s) if s == "x")
                && matches!(
                    &**pos,
                    Expression::Pos(pow2) if matches!(
                        &**pow2,
                        Expression::Pow(y, neg)
                        if matches!(&**y, Expression::Identifier(s) if s == "y")
                        && matches!(
                            &**neg,
                            Expression::Neg(z)
                            if matches!(&**z, Expression::Identifier(s) if s == "z"),
                        ),
                    ),
                ),
            ),
        ));
        assert_eq!(rest, "");

        let (rest, expr) = expr3("2^", true).unwrap();
        assert!(matches!(expr, Expression::Number(n) if n == frac(2, 1)));
        assert_eq!(rest, "^");
    }

    #[test]
    fn expr_test() {
        let (rest, e) = expr("x * y", false).unwrap();
        assert!(matches!(
            e,
            Expression::Mul(x, y)
            if matches!(&*x, Expression::Identifier(s) if s == "x")
            && matches!(&*y, Expression::Identifier(s) if s == "y"),
        ));
        assert_eq!(rest, "");

        let (rest, e) = expr("x * y+z*w\n+t", false).unwrap();
        assert!(matches!(
            e,
            Expression::Add(left, right)
            if matches!(
                &*left,
                Expression::Mul(x, y)
                if matches!(&**x, Expression::Identifier(s) if s == "x")
                && matches!(&**y, Expression::Identifier(s) if s == "y"),
            )
            && matches!(
                &*right,
                Expression::Mul(z, w)
                if matches!(&**z, Expression::Identifier(s) if s == "z")
                && matches!(&**w, Expression::Identifier(s) if s == "w"),
            ),
        ));
        assert_eq!(rest, "\n+t");

        let (rest, e) = expr("x - -3^4Ignore", true).unwrap();
        assert!(matches!(
            e,
            Expression::Sub(x, neg)
            if matches!(&*x, Expression::Identifier(s) if s == "x")
            && matches!(
                &*neg,
                Expression::Neg(pow)
                if matches!(
                    &**pow,
                    Expression::Pow(three, four)
                    if matches!(&**three, Expression::Number(n) if *n == frac(3, 1))
                    && matches!(&**four, Expression::Number(n) if *n == frac(4, 1)),
                ),
            ),
        ));
        assert_eq!(rest, "Ignore");

        let (rest, e) = expr("1 + 2 - 3 +\n 4xyz", true).unwrap();
        assert!(matches!(
            e,
            Expression::Add(sub, four)
            if matches!(
                &*sub,
                Expression::Sub(add, three)
                if matches!(
                    &**add,
                    Expression::Add(one, two)
                    if matches!(&**one, Expression::Number(n) if *n == frac(1, 1))
                    && matches!(&**two, Expression::Number(n) if *n == frac(2, 1)),
                )
                && matches!(&**three, Expression::Number(n) if *n == frac(3, 1)),
            )
            && matches!(&*four, Expression::Number(n) if *n == frac(4, 1)),
        ));
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn statement_test() {
        let (rest, stmt) = statement("x1 = y1 = z1#").unwrap();
        assert!(matches!(
            stmt,
            Statement::Assign(v)
            if matches!(
                &v[..],
                [x1, y1, z1]
                if matches!(&*x1, Expression::Identifier(s) if s == "x1")
                && matches!(y1, Expression::Identifier(s) if s == "y1")
                && matches!(z1, Expression::Identifier(s) if s == "z1"),
            ),
        ));
        assert_eq!(rest, "#");

        let (rest, stmt) = statement("-3").unwrap();
        assert!(matches!(
            &stmt,
            Statement::Evaluate(e)
            if matches!(
                e,
                Expression::Neg(three)
                if matches!(&**three, Expression::Number(n) if *n == frac(3, 1)),
            ),
        ));
        assert_eq!(rest, "");
    }

    #[test]
    fn program_test() {
        let (rest, prog) = program(
            "
            x = 1

            x
        ",
        )
        .unwrap();
        assert!(matches!(
            prog,
            Code { statements }
            if matches!(
                &statements[..],
                [Statement::Assign(assign), Statement::Evaluate(eval)]
                if matches!(
                    &assign[..],
                    [Expression::Identifier(x), Expression::Number(one)]
                    if x == "x"
                    && *one == frac(1, 1),
                )
                && matches!(
                    eval,
                    Expression::Identifier(x)
                    if x == "x",
                ),
            )
        ));
        assert_eq!(rest, "");
    }

    #[test]
    fn unicode_test() {
        let (rest, e) = expr("-\u{A0}\u{1680}_π'!", false).unwrap();
        assert!(matches!(
            e,
            Expression::Neg(pi)
            if matches!(&*pi, Expression::Identifier(s) if s == "_π'")),);
        assert_eq!(rest, "!");
    }
}
