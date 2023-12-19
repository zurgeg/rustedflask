use std::collections::HashMap;

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
    let template_name = "template.html.jinja2";
    let mut variables = HashMap::new();
    variables.insert("template_name", template_name.to_string());
    match render_template(template_name, variables) {
        Ok(content) => HTTPResponse::from(&*content),
        Err(why) => todo!(),
    }
}

fn main() {
    let mut app = App::new("example".to_string());
    app.route("/", main_route);
    app.route("/template", template_route);

    app.run("0.0.0.0:5000");
    panic!("Couldn't run");
}
