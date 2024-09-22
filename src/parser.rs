use core::str;

use winnow::ascii::dec_int;
use winnow::combinator::{cut_err, empty, fail, peek, preceded, repeat, terminated};
use winnow::error::{StrContext::*, StrContextValue::*};
use winnow::prelude::*;
use winnow::token::{any, take, take_until};
use winnow::{dispatch, BStr, Partial};

use crate::value::Value;

type Input<'i> = Partial<&'i BStr>;

const CRLF: &[u8] = b"\r\n";

pub fn new_input(input: &[u8]) -> Input {
    Partial::new(BStr::new(input))
}

pub fn value(input: &mut Input) -> PResult<Value> {
    dispatch! {
        peek(any);
        b'+' => simple_string,
        b'-' => simple_error,
        b':' => integer,
        b'$' => bulk_string,
        b'*' => array,
        _=> {
            // TODO: handle inline commands?
            cut_err(fail)
            .context(Expected(CharLiteral('+')))
            .context(Expected(CharLiteral('-')))
            .context(Expected(CharLiteral(':')))
            .context(Expected(CharLiteral('$')))
            .context(Expected(CharLiteral('*')))
        }
    }
    .parse_next(input)
}

#[inline]
fn line<'i>(input: &mut Input<'i>) -> PResult<&'i str> {
    terminated(take_until(0.., CRLF), CRLF)
        .try_map(|s| str::from_utf8(s))
        .parse_next(input)
}

#[inline]
fn length(input: &mut Input) -> PResult<i64> {
    terminated(dec_int, CRLF).parse_next(input)
}

fn simple_string(input: &mut Input) -> PResult<Value> {
    preceded(b'+', line)
        .map(|s| {
            if s == "OK" {
                Value::Ok
            } else {
                Value::SimpleString(s.into())
            }
        })
        .parse_next(input)
}

fn simple_error(input: &mut Input) -> PResult<Value> {
    preceded(b'-', line)
        .map(|s| Value::ServerError(s.into()))
        .parse_next(input)
}

fn integer(input: &mut Input) -> PResult<Value> {
    preceded(b':', terminated(dec_int, CRLF))
        .map(Value::Integer)
        .parse_next(input)
}

fn bulk_string(input: &mut Input) -> PResult<Value> {
    preceded(b'$', move |input: &mut Input| {
        let length: i64 = length.parse_next(input)?;
        if length < 0 {
            empty.map(|_| Value::Nil).parse_next(input)
        } else {
            terminated(take(length as usize), CRLF)
                .map(|s: &[u8]| Value::BulkString(s.into()))
                .parse_next(input)
        }
    })
    .parse_next(input)
}

fn array(input: &mut Input) -> PResult<Value> {
    preceded(b'*', move |input: &mut Input| {
        let length: i64 = length.parse_next(input)?;
        if length < 0 {
            empty.map(|_| Value::Nil).parse_next(input)
        } else {
            repeat(length as usize, value)
                .fold(Vec::new, |mut v: Vec<_>, elem: Value| {
                    v.push(elem);
                    v
                })
                .map(Value::Array)
                .parse_next(input)
        }
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::error::ErrMode;

    use super::*;

    #[test]
    fn test_ok() {
        let input = &mut new_input(b"+OK\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::Ok));
    }

    #[test]
    fn test_simple_string() {
        let input = &mut new_input(b"+Simple String\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::SimpleString("Simple String".into())));
    }

    #[test]
    fn test_simple_error() {
        let input = &mut new_input(b"-ERR: Simple Error\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::ServerError("ERR: Simple Error".into())));
    }

    #[test]
    fn test_bulk_string() {
        let input = &mut new_input(b"$4\r\n\xfa\xce\xfe\xed\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::BulkString(b"\xfa\xce\xfe\xed".into())));
    }

    #[test]
    fn test_bulk_string_nil() {
        let input = &mut new_input(b"$-1\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::Nil));
    }

    #[test]
    fn test_array() {
        let input = &mut new_input(b"*3\r\n$5\r\nhello\r\n$5\r\nworld\r\n*3\r\n:1\r\n:2\r\n:3\r\n");
        let result = value.parse_next(input);
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::BulkString("hello".into()),
                Value::BulkString("world".into()),
                Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3)
                ])
            ]))
        );
    }

    #[test]
    fn test_array_nil() {
        let input = &mut new_input(b"*-1\r\n");
        let result = value.parse_next(input);
        assert_eq!(result, Ok(Value::Nil));
    }

    #[test]
    fn test_incomplete() {
        let input = &mut new_input(b"*3\r\n");
        let result = value.parse_next(input);
        assert!(matches!(result, Err(ErrMode::Incomplete(_))));
    }
}
