use core::str;
use std::cmp::Ordering;

use Value::*;

use winnow::combinator::{dispatch, empty, fail, repeat, terminated, trace};
use winnow::prelude::*;
use winnow::token::{any, take, take_until};
use winnow::BStr;

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

pub fn parse(input: &mut &BStr) -> PResult<Value> {
    dispatch! {any;
        b'+' => simple_string,
        b'-' => simple_error,
        b':' => integer,
        b'$' => bulk_string,
        b'*' => array,
        // TODO: handle inline commands?
        _ => fail

    }
    .parse_next(input)
}

const CRLF: &[u8] = b"\r\n";

fn line<'a>(input: &mut &'a BStr) -> PResult<&'a str> {
    trace(
        "line",
        terminated(take_until(0.., CRLF), CRLF).try_map(|line| str::from_utf8(line)),
    )
    .parse_next(input)
}

fn simple_string(input: &mut &BStr) -> PResult<Value> {
    trace("simple_string", line.map(|s| SimpleString(s.into()))).parse_next(input)
}

fn simple_error(input: &mut &BStr) -> PResult<Value> {
    trace("simple_error", line.map(|s| SimpleError(s.into()))).parse_next(input)
}

fn integer(input: &mut &BStr) -> PResult<Value> {
    trace("integer", line.parse_to().map(Integer)).parse_next(input)
}

fn bulk_string<'a>(input: &mut &'a BStr) -> PResult<Value> {
    trace("bulk_string", move |input: &mut &'a BStr| {
        let length: i32 = line.parse_to().parse_next(input)?;
        if length < 0 {
            empty.value(Null).parse_next(input)
        } else {
            terminated(
                take(length as usize).map(|s: &[u8]| BulkString(s.into())),
                CRLF,
            )
            .parse_next(input)
        }
    })
    .parse_next(input)
}

fn array<'a>(input: &mut &'a BStr) -> PResult<Value> {
    trace("array", move |input: &mut &'a BStr| {
        let length: i32 = line.parse_to().parse_next(input)?;

        match length.cmp(&0) {
            Ordering::Less => empty.value(Null).parse_next(input),
            Ordering::Equal => empty.value(Value::Array(vec![])).parse_next(input),
            Ordering::Greater => repeat(length as usize, parse)
                .fold(Vec::new, |mut acc: Vec<_>, item| {
                    acc.push(item);
                    acc
                })
                .map(Array)
                .parse_next(input),
        }
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_string() {
        let result = parse(&mut BStr::new(b"+OK\r\n")).unwrap();
        assert_eq!(result, SimpleString("OK".into()));
    }

    #[test]
    fn test_simple_error() {
        let result = parse(&mut BStr::new(b"-ERR unknown command 'asdf'\r\n")).unwrap();
        assert_eq!(result, SimpleError("ERR unknown command 'asdf'".into()));
    }

    #[test]
    fn test_integer() {
        let input_expected = [
            (&b":0\r\n"[..], Integer(0)),
            (&b":-0\r\n"[..], Integer(0)),
            (&b":+0\r\n"[..], Integer(0)),
            (&b":1000\r\n"[..], Integer(1000)),
        ];

        for (input, expected) in input_expected {
            let result = parse(&mut BStr::new(input)).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_bulk_string() {
        let result = parse(&mut BStr::new(b"$5\r\nHELLO\r\n")).unwrap();
        assert_eq!(result, BulkString(b"HELLO".into()));
    }

    #[test]
    fn test_array() {
        let result = parse(&mut BStr::new(b"$5\r\nHELLO\r\n")).unwrap();
        assert_eq!(result, BulkString(b"HELLO".into()));
    }

    #[test]
    fn test_null_bulk_string() {
        let result = parse(&mut BStr::new(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n")).unwrap();
        assert_eq!(
            result,
            Array(vec![BulkString("hello".into()), BulkString("world".into())])
        );

        let result = parse(&mut BStr::new(b"*3\r\n:1\r\n:2\r\n:3\r\n")).unwrap();
        assert_eq!(result, Array(vec![Integer(1), Integer(2), Integer(3)]));
    }

    #[test]
    fn test_null_array() {
        let result = parse(&mut BStr::new(b"*-1\r\n")).unwrap();
        assert_eq!(result, Null);
    }
}
