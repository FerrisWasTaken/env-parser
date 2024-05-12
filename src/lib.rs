use chumsky::{
    error::Rich,
    extra,
    primitive::{any, choice, just},
    IterParser, Parser,
};

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub key: String,
    pub val: Value,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Comment(String),
    Assignment(Assignment),
}

fn comment<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> {
    just('#').ignore_then(any().filter(|c| c != &'\n').repeated().collect::<String>())
}

fn val<'a>() -> impl Parser<'a, &'a str, Value, extra::Err<Rich<'a, char>>> {
    let int = just("-")
        .or_not()
        .then(
            any()
                .filter(|i: &char| i.is_digit(10))
                .repeated()
                .at_least(1)
                .collect::<String>()
                .map(|i| i.parse::<i64>().unwrap()),
        )
        .map(|(neg, num)| {
            if neg.is_some() {
                Value::Int(num * -1)
            } else {
                Value::Int(num)
            }
        });
    let bool = choice((
        just("true").map(|_| Value::Bool(true)),
        just("false").map(|_| Value::Bool(false)),
    ));
    let string_with_dquote = just('"')
        .ignore_then(any().filter(|c| *c != '"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(|contents| Value::Str(contents));
    let free_string = any()
        .filter(|c: &char| c.is_alphanumeric())
        .repeated()
        .collect::<String>()
        .map(|content| Value::Str(content));
    choice((string_with_dquote, int, bool, free_string))
}

fn assignment<'a>() -> impl Parser<'a, &'a str, Assignment, extra::Err<Rich<'a, char>>> {
    let key = any()
        .filter(|c: &char| !c.is_whitespace() && *c != '=')
        .repeated()
        .at_least(1)
        .collect::<String>();
    just(' ')
        .repeated()
        .ignore_then(key)
        .then_ignore(just('='))
        .then(val().then_ignore(just(' ').repeated()))
        .map(|(key, val)| Assignment { key, val })
}

pub fn file<'a>() -> impl Parser<'a, &'a str, Vec<Stmt>, extra::Err<Rich<'a, char>>> {
    let stmt = choice((
        comment().map(|comm| Stmt::Comment(comm)),
        assignment().map(|assign| Stmt::Assignment(assign)),
    ));
    stmt.separated_by(just('\n'))
        .collect::<Vec<Stmt>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment().parse("# Hello world").into_result(),
            Ok(" Hello world".to_string())
        )
    }

    #[test]
    fn parse_val() {
        assert_eq!(val().parse("123").into_result(), Ok(Value::Int(123)));
        assert_eq!(
            val().parse(r#""Hello""#).into_result(),
            Ok(Value::Str("Hello".to_string()))
        );
        assert_eq!(
            val().parse(r#"Hello"#).into_result(),
            Ok(Value::Str("Hello".to_string()))
        );
        assert_eq!(val().parse("true").into_result(), Ok(Value::Bool(true)));
        assert_eq!(val().parse("false").into_result(), Ok(Value::Bool(false)));
    }
    #[test]
    fn parse_stmts() {
        assert_eq!(
            file().parse("a=b\n#Hello").into_result(),
            Ok(vec![
                Stmt::Assignment(Assignment {
                    key: "a".to_string(),
                    val: Value::Str("b".to_string())
                }),
                Stmt::Comment("Hello".to_string())
            ])
        )
    }
    #[test]
    fn parse_assignments() {
        assert_eq!(
            assignment().parse("a=b").into_result(),
            Ok(Assignment {
                key: "a".to_string(),
                val: Value::Str("b".to_string())
            })
        )
    }
}
