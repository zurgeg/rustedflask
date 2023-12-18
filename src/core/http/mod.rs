use std::{collections::HashMap, io::{Read, Write}, net::TcpStream};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

mod misc;
use misc::httpver_to_vecu8;

/// An HTTP status code
#[derive(FromPrimitive, Clone, Debug)]
pub enum HttpStatusCodes {
    Continue = 100,
    /// A notice that the server is switching protocols
    /// in response to an upgrade request
    SwitchingProtocols = 101,
    /// ## WebDAV Only!
    /// 
    /// The server is processing the request, but the client should wait for a response
    Processing = 102,
    /// idk http preloading is confusing
    EarlyHints = 103,

    /// IT WORKS!
    Ok = 200,
    /// A new resource was created
    Created = 201,
    /// The request has been accepted, but has not been acted upon
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    /// 200 OK but for Range requests
    PartialContent = 206,
    /// ## WebDAV Only!
    MultiStatus = 207,
    /// ## WebDAV Only!
    AlreadyReported = 208,
    /// ## HTTP Delta Encoding Only!
    IMUsed = 226,

    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    /// Deprecated for security concerns
    UseProxy = 305,
    Reserved306 = 306,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    BadRequest = 400,
    Unauthorized = 401,
    /// Experimental, reserve for future use
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    /// not in the http specs
    ChillOutMan = 420,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    Locked = 423,
    FailedDependency = 424,
    /// Experimental
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511
}

/// The reason why `Error::InvalidContentLength` was returned
#[derive(Debug)]
pub enum InvalidContentLengthReason {
    /// No `Content-Length` header was found
    MissingContentLength,
    /// The `Content-Length` error wasn't an integer, or couldn't be read as one.
    MalformedContentLength
}

/// An HTTP error
#[derive(Debug)]
pub enum Error {
    /// The HTTP message could not be read for an unknown reason
    UnreadableMessageError,
    /// The version was unreadable
    InvalidVersionError,
    /// The status code was unreadable, or couldn't be read as an int
    UnreadableStatusCode,
    /// The status code isn't in the HTTP standard
    UnknownStatusError,
    /// The `Content-Length` header couldn't be read
    InvalidContentLength(InvalidContentLengthReason),
    /// The stream could not be read
    StreamReadError,
    /// The socket didn't connect successfully
    CouldntConnect,
    /// The stream could not be written to
    CouldntSend,
    /// The recieved data was not HTTP (first five bytes were not `HTTP/`)
    NotHTTP
}

/// A response to an `HTTPRequest`
#[derive(Clone, Debug)]
pub struct HTTPResponse {
    /// Always `HTTP`
    pub httptag: Box<[u8]>,
    /// What HTTP version is in use?
    pub httpversion: (i32, i32),
    /// The status of the response
    pub statuscode: HttpStatusCodes,
    /// The human readable response reason, i.e., "OK" for 200
    pub reason: Box<[u8]>,
    /// Any headers the response has
    pub headers: HashMap<String, String>,
    /// The content
    pub content: Vec<u8>
}

/// An HTTP request
#[derive(Clone, Debug)]
pub struct HTTPRequest {
    /// The method of the request (`GET`, `POST`, etc.,)
    pub method: Vec<u8>,
    /// The path of the request
    pub path: Vec<u8>,
    /// Always `HTTP`
    pub httptag: Box<[u8]>,
    /// What HTTP version is in use?
    pub httpversion: (i32, i32),
    /// Any headers the request has
    pub headers: HashMap<String, String>,
    /// The content of the request
    pub content: Vec<u8>
}

impl Into<Vec<u8>> for HTTPRequest {
    /// Converts this request into an array of bytes (`u8`)
    /// # Examples
    /// ```
    /// # use rustedflask::core::http;
    /// # use std::collections::HashMap;
    /// # let mut headers = HashMap::new();
    /// # headers.insert("Host".to_string(), "example.com".to_string());
    /// # let request = http::HTTPRequest {
    /// #       method: Box::new(b"GET".to_owned()),
    /// #       path: Box::new(b"/".to_owned()),
    /// #       httptag: Box::new(b"HTTP".to_owned()),
    /// #       httpversion: (1, 1),
    /// #       headers: headers,
    /// #       content: b"".into(),
    /// # };
    /// let request_bytes: Vec<u8> = request.into();
    /// ```
    fn into(self) -> Vec<u8> {
        let mut out = Vec::new();
        // GET
        out.extend(self.method.iter());
        out.push(b' ');
        // GET /
        out.extend(self.path.iter());
        out.push(b' ');
        // GET / HTTP/
        out.extend(self.httptag.iter());
        out.push(b'/');
        // GET / HTTP/1
        out.extend(httpver_to_vecu8(self.httpversion));
        // Newline
        out.extend(b"\r\n".iter());
        // Headers
        for (header, val) in self.headers {
            out.extend(header.as_bytes());
            out.extend(b": ".iter());
            out.extend(val.as_bytes());
            out.extend(b"\r\n");
        }
        if self.content.len() != 0 {
            out.extend(self.content);
        };
        out.extend(b"\r\n");
        return out;
    }
}

impl HTTPRequest {
    /// Sends this request to the given `address` via TCP
    /// # Examples
    /// ```
    /// # use rustedflask::core::http;
    /// # use std::collections::HashMap;
    /// # let mut headers = HashMap::new();
    /// # headers.insert("Host".to_string(), "example.com".to_string());
    /// # let mut request = http::HTTPRequest {
    /// #       method: Box::new(b"GET".to_owned()),
    /// #       path: Box::new(b"/".to_owned()),
    /// #       httptag: Box::new(b"HTTP".to_owned()),
    /// #       httpversion: (1, 1),
    /// #       headers: headers,
    /// #       content: b"".into(),
    /// # };
    /// // Watch out! You need the port
    /// request.send_to("example.com:80".into());
    /// ```
    pub fn send_to(&mut self, address: String) -> Result<HTTPResponse, Error> {
        let stream = TcpStream::connect(address);
        if stream.is_err() {
            return Err(Error::CouldntConnect);
        };

        let mut unwrapped_stream = stream.unwrap();
        let send_buffer = &mut [0 as u8; 1];
        for byte in Into::<Vec<u8>>::into(self.to_owned()) {
            send_buffer[0] = byte;
            let err = unwrapped_stream.write(send_buffer);
            if err.is_err() {
                return Err(Error::CouldntSend);
            };
            assert_eq!(err.unwrap(), 1 as usize);
        };
        return HTTPResponse::read_http_response(&mut unwrapped_stream);
    }

    /// Reads an HTTP request from `stream` into an HTTPRequest
    pub fn read_http_request(stream: &mut impl Read) -> Result<HTTPRequest, Error> {
        let mut method_string = String::new();
        let meth_read_buffer = &mut [0 as u8; 1];
        stream.read(meth_read_buffer);
        while meth_read_buffer[0] != 0x20 {
            method_string.push(meth_read_buffer[0].into());
            stream.read(meth_read_buffer);
        }

        let mut path_string = String::new();
        let path_read_buffer = &mut [0 as u8; 1];
        stream.read(path_read_buffer);
        while path_read_buffer[0] != 0x20 {
            path_string.push(path_read_buffer[0].into());
            stream.read(path_read_buffer);
        }

        let method = method_string.into_bytes();
        let path = path_string.into_bytes();

        // read the HTTP thing
        let httptag: &mut [u8; 5] = &mut [0 as u8; 5];
        let mut err = stream.read(httptag);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        if httptag != b"HTTP/" {
            return Err(Error::NotHTTP);
        };
        let http_version_bytes = &mut [0 as u8; 3];
        err = stream.read(http_version_bytes);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        if http_version_bytes[1] != b'.' {
            return Err(Error::InvalidVersionError);
        };
        let http_major = char::try_from(http_version_bytes[0]).unwrap().to_string().parse::<i32>();
        if http_major.is_err() {
            return Err(Error::InvalidVersionError);
        };
        let http_minor = char::try_from(http_version_bytes[2]).unwrap().to_string().parse::<i32>();
        if http_minor.is_err() {
            return Err(Error::InvalidVersionError);
        };
        let httpversion = (http_major.unwrap(), http_minor.unwrap());

        let _ = stream.read(&mut [0 as u8; 1]);

        let mut headers = HashMap::<String, String>::new();

        loop {
            let mut header_key = String::new();
            let mut header_val = String::new();
            let cur_char = &mut [0 as u8; 1];
            err = stream.read(cur_char);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            if cur_char[0] == b'\r' {
                break
            }
            while cur_char[0] != b':' {
                header_key.push(cur_char[0].into());
                err = stream.read(cur_char);
                if err.is_err() {
                    return Err(Error::StreamReadError)
                }
            }
            let _ = stream.read(cur_char);
            err = stream.read(cur_char);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            while cur_char[0] != b'\r' {
                header_val.push(cur_char[0].into());
                err = stream.read(cur_char);
                if err.is_err() {
                    return Err(Error::StreamReadError)
                }
            }
            let _ = stream.read(cur_char);
            headers.insert(header_key, header_val);
        };
        // todo finish
        let mut l_read = 0;
        let mut content = Vec::<u8>::new();
        if headers.contains_key("Content-Length") {
            let string_content_length = headers["Content-Length"].parse();
            if string_content_length.is_err(){
                return Err(Error::InvalidContentLength(InvalidContentLengthReason::MalformedContentLength));
            };
            let content_length = string_content_length.unwrap();
            while l_read < content_length {
                l_read += 1;
                let tempbuf = &mut [0 as u8; 1];
                err = stream.read(tempbuf);
                if err.is_err() {
                    return Err(Error::StreamReadError)
                }
                content.push(tempbuf[0]);
            };
        };
        return Ok(HTTPRequest {
            method,
            path,
            httptag: Box::new(*httptag),
            httpversion,
            headers,
            content
        });
    }
}

impl Into<Vec<u8>> for HTTPResponse {
    /// Sends this request to the given `address` via TCP
    /// # Examples
    /// ```
    /// # use rustedflask::core::http;
    /// # use std::collections::HashMap;
    /// # let mut headers = HashMap::new();
    /// # headers.insert("Host".to_string(), "example.com".to_string());
    /// # let mut response = http::HTTPResponse {
    /// #   httptag: Box::new(b"HTTP".to_owned()),
    /// #   httpversion: (1, 1),
    /// #   statuscode: http::HttpStatusCodes::Ok,
    /// #   reason: Box::new(b"OK".to_owned()),
    /// #   headers: headers,
    /// #   content: b"".into()
    /// # };
    /// let response_bytes: Vec<u8> = response.into();
    /// ```
    fn into(self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(self.httptag.iter());
        out.push(b'/');
        out.extend(httpver_to_vecu8(self.httpversion));
        out.push(b' ');
        out.extend(Vec::<u8>::from((self.statuscode as i32).to_string()).iter());
        out.push(b' ');
        out.extend(self.reason.iter());
        out.extend(b"\r\n".iter());
        // Headers
        for (header, val) in self.headers {
            out.extend(header.as_bytes());
            out.extend(b": ".iter());
            out.extend(val.as_bytes());
            out.extend(b"\r\n");
        }
        out.extend(b"\r\n");
        if self.content.len() != 0 {
            out.extend(self.content);
        };
        out.extend(b"\r\n");
        return out;
    }
}

impl From<&str> for HTTPResponse {
    fn from(value: &str) -> Self {
        let mut headers = HashMap::<String, String>::new();
        headers.insert("Content-Length".into(), value.len().to_string());
        return HTTPResponse {
            httptag: Box::new(b"HTTP".to_owned()),
            httpversion: (1, 1),
            statuscode: HttpStatusCodes::Ok,
            reason: Box::new(b"OK".to_owned()),
            headers,
            content: value.to_string().into_bytes()
        }
    }
}

impl HTTPResponse {
    /// Reads an HTTP response from `stream` into an HTTPResponse
    pub fn read_http_response(stream: &mut impl Read) -> Result<HTTPResponse, Error> {
        // read the HTTP thing
        let http_tag: &mut [u8; 5] = &mut [0 as u8; 5];
        let mut err = stream.read(http_tag);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        if http_tag != b"HTTP/" {
            return Err(Error::NotHTTP);
        };
        let http_version_bytes = &mut [0 as u8; 3];
        err = stream.read(http_version_bytes);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        if http_version_bytes[1] != b'.' {
            return Err(Error::InvalidVersionError);
        };
        let http_major = char::try_from(http_version_bytes[0]).unwrap().to_string().parse::<i32>();
        if http_major.is_err() {
            return Err(Error::InvalidVersionError);
        };
        let http_minor = char::try_from(http_version_bytes[2]).unwrap().to_string().parse::<i32>();
        if http_minor.is_err() {
            return Err(Error::InvalidVersionError);
        };
        let http_version = (http_major.unwrap(), http_minor.unwrap());

        let statuscode = &mut [0 as u8; 3];
        // read the space between the version number and the status code 
        err = stream.read(&mut [0 as u8; 1]);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        // get the 3 digit status code
        err = stream.read(statuscode);
        if err.is_err() {
            return Err(Error::StreamReadError)
        }
        let mut status_string = String::new();
        for character in statuscode {
            status_string.push(char::from(character.to_owned()));
        }
        let status_int = status_string.parse::<i32>();
        if status_int.is_err() {
            return Err(Error::UnreadableStatusCode);
        };
        let status: Option<HttpStatusCodes> = HttpStatusCodes::from_i32(status_int.unwrap());
        if status.is_none() {
            return Err(Error::UnknownStatusError)
        };
        let nl_buf = &mut [0 as u8; 1];
        let mut reason = Vec::new();
        err = stream.read(nl_buf);
        while nl_buf[0] != b'\r' {
            reason.push(nl_buf[0]);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            err = stream.read(nl_buf);
        };
        let _ = stream.read(nl_buf);
        let mut headers = HashMap::<String, String>::new();

        loop {
            let mut header_key = String::new();
            let mut header_val = String::new();
            let cur_char = &mut [0 as u8; 1];
            err = stream.read(cur_char);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            if cur_char[0] == b'\r' {
                break
            }
            while cur_char[0] != b':' {
                header_key.push(cur_char[0].into());
                err = stream.read(cur_char);
                if err.is_err() {
                    return Err(Error::StreamReadError)
                }
            }
            let _ = stream.read(cur_char);
            err = stream.read(cur_char);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            while cur_char[0] != b'\r' {
                header_val.push(cur_char[0].into());
                err = stream.read(cur_char);
                if err.is_err() {
                    return Err(Error::StreamReadError)
                }
            }
            let _ = stream.read(cur_char);
            headers.insert(header_key, header_val);
        };
        // todo finish
        let mut l_read = 0;
        if !headers.contains_key("Content-Length") {
            return Err(Error::InvalidContentLength(InvalidContentLengthReason::MissingContentLength));
        }
        let string_content_length = headers["Content-Length"].parse();
        if string_content_length.is_err(){
            return Err(Error::InvalidContentLength(InvalidContentLengthReason::MalformedContentLength));
        };
        let content_length = string_content_length.unwrap();
        let mut content = Vec::<u8>::new();
        while l_read < content_length {
            l_read += 1;
            let tempbuf = &mut [0 as u8; 1];
            err = stream.read(tempbuf);
            if err.is_err() {
                return Err(Error::StreamReadError)
            }
            content.push(tempbuf[0]);
        };
        return Ok(HTTPResponse {
            httptag: Box::new(*http_tag),
            httpversion: http_version,
            reason: reason.into(),
            statuscode: status.unwrap(),
            headers,
            content
        });
    }
}