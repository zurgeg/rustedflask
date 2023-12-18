use rustedflask::{
    flask::App,
    jinja::render_template,
    core::http::{
        HTTPRequest,
        HTTPResponse
    }
};

fn main_route(request: HTTPRequest) -> HTTPResponse {
    "Hello, world!".into()
}

fn template_route(request: HTTPRequest) -> HTTPResponse {
    render_template(file, variables)
}

fn main() {
    println!("Hello, world!");
}
