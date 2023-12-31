#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
//! Internet programming library for Rust

/// Core APIs
pub mod core;
/// A Flask-based API for Rust
pub mod flask;

#[cfg(feature = "jinja")]
/// A Jinja parser and renderer for Rust
pub mod jinja;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Read};

    use crate::core::misc::ReadableVec;

    #[cfg(feature = "jinja")]
    use crate::jinja::render_template_string;

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
        let buf = &mut [0_u8; 3];
        let mut readablevec = ReadableVec {
            vector: &mut vec.clone(),
        };
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
            content: b"".into(),
        };
        let mut resp_bytes: Vec<u8> = example_response.into();
        let resp_parsed = core::http::HTTPResponse::read_http_response(&mut ReadableVec {
            vector: &mut resp_bytes,
        });
        if resp_parsed.is_err() {
            return Err(resp_parsed.unwrap_err());
        }
        Ok(())
    }

    #[test]
    #[cfg(feature = "jinja")]
    fn test_jinja_variable() -> Result<(), jinja::JinjaError> {
        let template = "{{ variable }}".to_string();
        let mut variables = HashMap::<&str, String>::new();
        variables.insert("variable", "works".to_string());
        let rendered = match render_template_string(template, variables, None) {
            Err(why) => return Err(why),
            Ok(response) => response,
        };
        assert_eq!(rendered, "works".to_string());
        Ok(())
    }
    #[test]
    #[cfg(feature = "jinja")]
    fn test_jinja_function_no_args() -> Result<(), jinja::JinjaError> {
        fn test_function(args: Vec<String>) -> String {
            "works".to_string()
        }
        let template = "{{ function() }}".to_string();
        let mut functions: HashMap<&str, jinja::JinjaFunction> = HashMap::new();
        functions.insert("function", test_function);
        let rendered = match render_template_string(template, HashMap::new(), Some(functions)) {
            Err(why) => return Err(why),
            Ok(response) => response,
        };
        assert_eq!(rendered, "works".to_string());
        Ok(())
    }

    #[test]
    #[cfg(feature = "jinja")]
    fn test_jinja_function_args() -> Result<(), jinja::JinjaError> {
        fn test_function(args: Vec<String>) -> String {
            let mut return_val = args[0].clone();
            return_val.extend(args[1].chars());
            return_val.extend(args[2].chars());
            return_val.extend(args[3].chars());
            return_val
        };
        let template = r#"{{ function("works", "blah","hah", variable) }}"#.to_string();
        let mut functions: HashMap<&str, jinja::JinjaFunction> = HashMap::new();
        functions.insert("function", test_function);

        let mut variables = HashMap::new();
        variables.insert("variable", "gah".to_string());
        let rendered = match render_template_string(template, variables, Some(functions)) {
            Err(why) => return Err(why),
            Ok(response) => response,
        };
        assert_eq!(rendered, "worksblahhahgah".to_string());
        Ok(())
    }
}
