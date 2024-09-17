use core::str;

use winnow::ascii::dec_int;
use winnow::combinator::{dispatch, empty, fail, peek, preceded, repeat, terminated, trace};
use winnow::token::{any, take, take_until};
use winnow::BStr;
use winnow::{prelude::*, Partial};

use Value::*;

const CRLF: &[u8] = b"\r\n";

type Input<'s> = Partial<&'s BStr>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    // +OK\r\n
    SimpleString(String),
    // -ERR message\r\n
    SimpleError(String),
    // :[<+|->]<value>\r\n
    Integer(i64),
    // $<length>\r\n<data>\r\n
    BulkString(Vec<u8>),
    // *<number-of-elements>\r\n<element-1>...<element-n>
    Array(Vec<Value>),
    // $-1\r\n or *-1\r\n
    Null,
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
        preceded(b'-', line).map(|s| SimpleError(s.into())),
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
        assert_eq!(result, Ok(SimpleError("ERR unknown command 'asdf'".into())));
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
