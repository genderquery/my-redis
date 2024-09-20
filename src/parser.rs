use core::str;
use std::fmt;

use winnow::ascii::dec_int;
use winnow::combinator::{dispatch, empty, fail, peek, preceded, repeat, terminated, trace};
use winnow::token::{any, take, take_until};
use winnow::BStr;
use winnow::{prelude::*, Partial};

use Value::*;

const CRLF: &[u8] = b"\r\n";

type Input<'s> = Partial<&'s BStr>;

#[derive(PartialEq, Eq, Clone)]
pub enum Value {
    SimpleString(String),
    ServerError(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Value>),
    Null,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_value_debug(f, self, 0)
    }
}

fn write_value_debug(f: &mut fmt::Formatter<'_>, v: &Value, indent: usize) -> fmt::Result {
    match v {
        Value::Null => writeln!(f, "(nil)"),
        Value::SimpleString(s) => writeln!(f, "\"{s}\""),
        Value::ServerError(s) => writeln!(f, "(error) {s}"),
        Value::Integer(i) => writeln!(f, "(integer) {i}"),
        Value::BulkString(b) => writeln!(f, "\"{}\"", b.escape_ascii()),
        Value::Array(v) => {
            if v.is_empty() {
                writeln!(f, "(empty array)")?;
            } else {
                let num_width = (v.len() as f64).log10().floor() as usize + 1;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{:>indent$}", "")?;
                    }
                    write!(f, "{i:>num_width$}) ", i = i + 1)?;
                    write_value_debug(f, v, indent + num_width + 2)?
                }
            }
            Ok(())
        }
    }
}

pub fn value(input: &mut Input) -> PResult<Value> {
    trace(
        "value",
        dispatch!(peek(any);
            b'+' => simple_string,
            b'-' => simple_error,
            b':' => integer,
            b'$' => bulk_string,
            b'*' => array,
            // TODO: handle inline commands?
            _ => fail
        ),
    )
    .parse_next(input)
}

fn line<'s>(input: &mut Input<'s>) -> PResult<&'s str> {
    terminated(take_until(0.., CRLF), CRLF)
        .try_map(|s| str::from_utf8(s))
        .parse_next(input)
}

fn simple_string(input: &mut Input) -> PResult<Value> {
    trace(
        "simple_string",
        preceded(b'+', line).map(|s| SimpleString(s.into())),
    )
    .parse_next(input)
}

fn simple_error(input: &mut Input) -> PResult<Value> {
    trace(
        "simple_error",
        preceded(b'-', line).map(|s| ServerError(s.into())),
    )
    .parse_next(input)
}

fn integer(input: &mut Input) -> PResult<Value> {
    trace(
        "integer",
        preceded(b':', terminated(dec_int, CRLF)).map(Integer),
    )
    .parse_next(input)
}

fn bulk_string(input: &mut Input) -> PResult<Value> {
    trace(
        "bulk_string",
        preceded(b'$', move |input: &mut Input| {
            let length: i64 = terminated(dec_int, CRLF).parse_next(input)?;
            if length < 0 {
                empty.value(Null).parse_next(input)
            } else {
                terminated(
                    take(length as usize).map(|s: &[u8]| BulkString(s.into())),
                    CRLF,
                )
                .parse_next(input)
            }
        }),
    )
    .parse_next(input)
}

fn array(input: &mut Input) -> PResult<Value> {
    trace(
        "array",
        preceded(b'*', move |input: &mut Input| {
            let length: i64 = terminated(dec_int, CRLF).parse_next(input)?;
            if length < 0 {
                empty.value(Null).parse_next(input)
            } else {
                repeat(length as usize, value)
                    .fold(Vec::new, |mut v: Vec<_>, elem: Value| {
                        v.push(elem);
                        v
                    })
                    .map(Array)
                    .parse_next(input)
            }
        }),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::error::ErrMode;

    use super::*;

    fn input_from_bstr(s: &[u8]) -> Input {
        Partial::new(BStr::new(s))
    }

    #[test]
    fn test_simple_string() {
        let result = value(&mut input_from_bstr(b"+OK\r\n"));
        assert_eq!(result, Ok(SimpleString("OK".into())));
    }

    #[test]
    fn test_simple_error() {
        let result = value(&mut input_from_bstr(b"-ERR unknown command 'asdf'\r\n"));
        assert_eq!(result, Ok(ServerError("ERR unknown command 'asdf'".into())));
    }

    #[test]
    fn test_integer() {
        let input_expected = [
            (&b":0\r\n"[..], Integer(0)),
            (&b":1000\r\n"[..], Integer(1000)),
            (&b":+1000\r\n"[..], Integer(1000)),
            (&b":-1000\r\n"[..], Integer(-1000)),
        ];

        for (input, expected) in input_expected {
            let result = value(&mut input_from_bstr(input));
            assert_eq!(result, Ok(expected));
        }
    }

    #[test]
    fn test_bulk_string() {
        let result = value(&mut input_from_bstr(b"$5\r\nHELLO\r\n"));
        assert_eq!(result, Ok(BulkString(b"HELLO".into())));
    }

    #[test]
    fn test_null_bulk_string() {
        let result = value(&mut input_from_bstr(b"$-1\r\n"));
        assert_eq!(result, Ok(Null));
    }

    #[test]
    fn test_array() {
        let result = value(&mut input_from_bstr(
            b"*5\r\n$5\r\nhello\r\n$5\r\nworld\r\n:1\r\n:2\r\n:3\r\n",
        ));
        assert_eq!(
            result,
            Ok(Array(vec![
                BulkString("hello".into()),
                BulkString("world".into()),
                Integer(1),
                Integer(2),
                Integer(3)
            ]))
        );
    }

    #[test]
    fn test_null_array() {
        let result = value(&mut input_from_bstr(b"*-1\r\n"));
        assert_eq!(result, Ok(Null));
    }

    #[test]
    fn test_partial() {
        let result = value(&mut input_from_bstr(b"*3\r\n"));
        assert!(matches!(result, Err(ErrMode::Incomplete(_))));
    }
}
