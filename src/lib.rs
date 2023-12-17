pub mod core;
pub mod flask;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Read};

    use crate::core::misc::ReadableVec;

    use super::*;

    #[test]
    fn test_send() -> Result<(), core::http::Error> {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "example.com".to_string());
        let mut example_request = core::http::HTTPRequest {
            method: b"GET".to_vec(),
            path: b"/".to_vec(),
            httptag: Box::new(b"HTTP".to_owned()),
            httpversion: (1, 1),
            headers: headers,
            content: b"".into(),
        };
        example_request.send_to("example.com:80".to_string())?;
        return Ok(());
    }
    #[test]
    fn test_readablevec() -> Result<(), std::io::Error> {
        let vec = vec![b'f', b'o', b'o'];
        let buf = &mut [0 as u8; 3];
        let mut readablevec = ReadableVec { vector: &mut vec.clone() };
        readablevec.read(buf)?;
        assert_eq!(vec[0], buf[0]);
        assert_eq!(vec[1], buf[1]);
        assert_eq!(vec[2], buf[2]);
        return Ok(());
    }
    #[test]
    fn test_parse() -> Result<(), core::http::Error> {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "example.com".to_string());
        headers.insert("Content-Length".to_string(), "0".to_string());
        let example_response = core::http::HTTPResponse {
            httptag: Box::new(b"HTTP".to_owned()),
            httpversion: (1, 1),
            reason: Box::new(b"OK".to_owned()),
            statuscode: core::http::HttpStatusCodes::NoContent,
            headers: headers,
            content: b"".into()
        };
        let mut resp_bytes: Vec<u8> = example_response.into();
        let resp_parsed = core::http::HTTPResponse::read_http_response(&mut ReadableVec{vector: &mut resp_bytes});
        if resp_parsed.is_err() {
            return Err(resp_parsed.unwrap_err());
        }
        return Ok(());
    }
}