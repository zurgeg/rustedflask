use crate::core::http::{HTTPRequest, HTTPResponse, HttpStatusCodes};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
};

/// A callback function for when a route is accessed
pub type RouteFn = fn(request: HTTPRequest) -> HTTPResponse;

#[derive(Clone)]
struct Route {
    pub path: String,
    pub func: RouteFn,
}

/// An app (similar to Python's `flask.Flask`)
pub struct App {
    /// The name of this app
    pub name: String,
    routes: Vec<Route>,
}

/// Could not bind to the given address
pub struct CantBind;

impl App {
    /// Makes a new app
    ///
    /// Equivalent to
    /// ```python
    /// app = Flask("name")
    /// ````
    ///
    /// # Examples
    /// ```rust
    /// # use rustedflask::flask::App;
    /// let app = App::new("name".to_string());
    /// ```
    pub fn new(name: String) -> App {
        App {
            name,
            routes: Vec::new(),
        }
    }

    fn handle(&mut self, request: HTTPRequest, mut client: TcpStream) {
        let proper_request_path = request.path.to_vec();
        let route_string = String::from_utf8(proper_request_path);

        if route_string.is_err() {
            return;
        }

        let route = self.find_route_for_path(route_string.clone().unwrap().as_str());

        if route.is_none() {
            let notfoundroute_wrapped = self.find_route_for_path("!404");
            if let Some(notfoundroute) = notfoundroute_wrapped {
                thread::spawn(move || {
                    let response: Vec<u8> = (notfoundroute.func)(request).into();
                    let buf = &mut [0_u8];
                    for byte in response {
                        buf[0] = byte;
                        let err = client.write(buf);
                        if err.is_err() {
                            panic!("{:?}", err.unwrap_err())
                        };
                    }
                });
            } else {
                let mut response_http = HTTPResponse::from("404 Not Found");
                response_http.statuscode = HttpStatusCodes::NotFound;
                response_http.reason = Box::new(b"Not Found".to_owned());
                let response: Vec<u8> = response_http.into();
                let buf = &mut [0_u8];
                for byte in response {
                    buf[0] = byte;
                    let err = client.write(buf);
                    if err.is_err() {
                        panic!("{:?}", err.unwrap_err())
                    };
                }
            };
            return;
        };

        thread::spawn(move || {
            let response: Vec<u8> = (route.unwrap().func)(request).into();
            let buf = &mut [0_u8];
            for byte in response {
                buf[0] = byte;
                let err = client.write(buf);
                if err.is_err() {
                    panic!("{:?}", err.unwrap_err())
                }
            }
        });
    }

    fn find_route_for_path(&mut self, path: &str) -> Option<Route> {
        for route in &self.routes {
            if route.path == *path {
                return Some(route.clone());
            };
        }
        None
    }

    /// Creates a route for `path`, calling `func` when
    /// the route is accessed
    pub fn route(&mut self, path: &str, func: RouteFn) {
        self.routes.push(Route {
            path: path.to_string(),
            func,
        })
    }

    /// Runs the (debug!) webserver
    pub fn run(&mut self, bind_address: &str) -> CantBind {
        let serversock_wrapped = TcpListener::bind(bind_address);

        if serversock_wrapped.is_err() {
            return CantBind;
        };
        let serversock = serversock_wrapped.unwrap();

        println!("OK. Server active on addres: {}", bind_address);

        loop {
            // await for a client
            let mut client = serversock.accept();
            if client.is_ok() {
                let request = HTTPRequest::read_http_request(&mut client.as_mut().unwrap().0);
                if request.is_err() {
                    println!("Can't read request... {:?}", request.unwrap_err());
                    continue;
                };
                self.handle(request.unwrap(), client.unwrap().0);
            }
        }
    }
}
