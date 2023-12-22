use std::collections::HashMap;

use rustedflask::{
    core::http::{HTTPRequest, HTTPResponse, HttpStatusCodes},
    flask::App,
    jinja::render_template,
};

fn main_route(_request: HTTPRequest) -> HTTPResponse {
    "Hello, world!".into()
}

fn template_route(_request: HTTPRequest) -> HTTPResponse {
    let template_name = "template.html.jinja2";
    let mut variables = HashMap::new();
    variables.insert("template_name", template_name.to_string());
    match render_template(template_name, variables, None) {
        Ok(content) => HTTPResponse::from(&*content),
        // Build an error page
        Err(why) => HTTPResponse::new()
            .with_statuscode(
                HttpStatusCodes::InternalServerError,
                Box::new(b"Internal Server Error".to_owned()),
            )
            .with_content(format!("{:?}", why).into()),
    }
}

fn inheritance_route(_request: HTTPRequest) -> HTTPResponse {
    match render_template("inheritance.html.jinja2", HashMap::new(), None) {
        Ok(content) => HTTPResponse::from(&*content),
        Err(why) => HTTPResponse::new()
            .with_statuscode(
                HttpStatusCodes::InternalServerError,
                Box::new(b"Internal Server Error".to_owned()),
            )
            .with_content(format!("{:?}", why).into()),
    }
}

fn route_you_can_only_post_to(_request: HTTPRequest) -> HTTPResponse {
    "You can only use the POST method to access this route".into()
}

fn main() {
    let mut app = App::new("example".to_string());
    app.route("/", main_route);
    app.route("/template", template_route);
    app.route("/inheritance", inheritance_route);
    app.route_with_allowed_methods(
        "/postonly",
        route_you_can_only_post_to,
        vec!["POST".to_string()],
    );

    app.run("0.0.0.0:5000");
    panic!("Couldn't run");
}
