mod consts;

use regex::Regex;

use std::{collections::HashMap, path::Path, fs::File, io::Read};

use crate::core::http::HTTPResponse;

pub enum JinjaError {
    TemplateNotFound,
    Other(String),
}

pub fn render_template_string(template: String, variables: HashMap<&str, String>) -> Result<HTTPResponse, JinjaError> {
    let simple_variable = Regex::new(consts::REPLACE);

    Err(JinjaError::Other("not implemented yet".to_string()))
}

pub fn render_template(file: &str, variables: HashMap<&str, String>) -> Result<HTTPResponse, JinjaError> {
    // Variables are <&str, String> because the key is more likely to be
    // a string const, and the value is more likely to be dynamically generated
    let fpath = Path::new(file);
    let mut opened_file = match File::open(fpath) {
        Err(why) => return Err(JinjaError::Other(format!("can't open file, error: {}", why))),
        Ok(file) => file
    };

    let mut contents = String::new();

    match opened_file.read_to_string(&mut contents) {
        Err(why) => return Err(JinjaError::Other(format!("couldn't read file, error: {}", why))),
        Ok(_) => return render_template_string(contents, variables),
    }
}