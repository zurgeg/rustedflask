use std::{collections::HashMap, process::exit, sync::{Arc, Mutex}};

use rustedflask::{
    core::http::{HTTPRequest, HTTPResponse, HttpStatusCodes},
    flask::App,
    jinja::{render_template, JinjaState}, wrap_context,
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

// This takes advantage of template caching, which speeds it up
//
// This comes at the disadvantage of stopping other cached routes from accessing
// the state until it is done
// 
// Additionally, routes taking advantage of caching should do strict error checking
// to prevent the mutex from being poisoned
fn template_route_cached(ctx_unlocked: Arc<Mutex<JinjaState>>, _request: HTTPRequest) -> HTTPResponse {
    let mut ctx = ctx_unlocked.lock().unwrap();
    let template_name = "template.html.jinja2";
    let mut variables = HashMap::new();
    variables.insert("template_name", template_name.to_string());
    match ctx.render_template(template_name, variables, None) {
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

fn shutdown(_request: HTTPRequest) -> HTTPResponse {
    exit(0);
}

fn main() {
    let mut app = App::new("example".to_string());
    let jinja_state = JinjaState::new();
    let ctx = Arc::new(Mutex::new(jinja_state));
    app.route("/", main_route);
    app.route("/template", template_route);
    app.route("/inheritance", inheritance_route);
    app.route("/shutdown", shutdown);
    app.route_with_allowed_methods(
        "/postonly",
        route_you_can_only_post_to,
        vec!["POST".to_string()],
    );

    app.route("/caching", wrap_context!(
        template_route_cached,
        ctx
    ));

    app.run("0.0.0.0:5000");
    panic!("Couldn't run");
}
