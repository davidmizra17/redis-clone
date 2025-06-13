use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};
use bytes::BytesMut;
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            _ => panic!("Unsupported value for serialization"),
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let byte_count = self.stream.read_buf(&mut self.buffer).await?;

        if byte_count == 0 {
            return Ok(None);
        }

        let(v, _) = parse_message(self.buffer.split())?;
        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        unimplemented!() 
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '$' => parse_bulk_string(buffer),
        // '*' => parse_array(buffer),
        _ => Err(anyhow::anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_clrf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Ok((Value::SimpleString(string), len + 1));
    }
    return Err(anyhow::anyhow!("Invalid string {:?}", buffer));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_string_len, bytes_consumed) = match read_until_clrf(&buffer[1..]) {
        Some((line, len)) => {
            let bulk_string_len = parse_int(line)?;
            (bulk_string_len, len + 1)
        }
        None => {
            return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
        }
    };

    let end_of_bulk_str = bytes_consumed + bulk_string_len as usize;
    let total_parsed = end_of_bulk_str + 2;

    Ok((Value::BulkString(String::from_utf8(buffer[bytes_consumed..end_of_bulk_str].to_vec())?), total_parsed))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_length, mut bytes_consumed) = match read_until_clrf(&buffer[1..]) {
        Some((line, len)) => {
            let array_length = parse_int(line)?;

        (array_length, len + 1)
        }
        None => {
            return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
        }
    };   
    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(array_item);
        bytes_consumed += len;
    };
    return Ok((Value::Array(items), bytes_consumed))
}

fn read_until_clrf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    return None;
}
fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}


#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_parse_simple_string_valid() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(b"+OK\r\n");
        let result = parse_simple_string(buffer);
        assert!(result.is_ok());
        let (value, len) = result.unwrap();
        assert_eq!(len, 5); // "+OK\r\n" is 5 bytes long
        match value {
            Value::SimpleString(s) => assert_eq!(s, "OK"),
            _ => panic!("Expected SimpleString"),
        }
    }

    #[test]
    fn test_parse_simple_string_empty() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(b"+\r\n");
        let result = parse_simple_string(buffer);
        assert!(result.is_ok());
        let (value, len) = result.unwrap();
        assert_eq!(len, 3); // "+\r\n" is 3 bytes long
        match value {
            Value::SimpleString(s) => assert_eq!(s, ""),
            _ => panic!("Expected SimpleString"),
        }
    }

    #[test]
    fn test_parse_simple_string_no_crlf() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(b"+Hello World");
        let result = parse_simple_string(buffer);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid string"));
    }

    #[test]
    fn test_parse_simple_string_with_extra_data() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(b"+Message\r\nExtraData");
        let result = parse_simple_string(buffer);
        assert!(result.is_ok());
        let (value, len) = result.unwrap();
        // len should represent the consumed part, which is "+Message\r\n"
        assert_eq!(len, 10);
        match value {
            Value::SimpleString(s) => assert_eq!(s, "Message"),
            _ => panic!("Expected SimpleString"),
        }
    }

    // #[test]
    // #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: FromUtf8Error { bytes: [194], valid_up_to: 0, error_len: Some(1) }")]
    // fn test_parse_simple_string_invalid_utf8() {
    //     let mut buffer = BytesMut::new();
    //     // 0xC2 is a lead byte for a 2-byte sequence in UTF-8, but it's incomplete
    //     // if not followed by a second byte in the range 0x80-0xBF.
    //     // This simulates an invalid UTF-8 sequence.
    //     buffer.extend_from_slice(b"+\xC2\r\n");
    //     let _ = parse_simple_string(buffer).unwrap(); // This should panic
    // }
}