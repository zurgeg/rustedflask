mod consts;

use regex::Regex;

use std::{collections::HashMap, fs::File, io::Read, path::Path};

/// A function that can be passed to a Jinja template
/// ### Warning
/// Unlike in Python's Jinja, where functions are written like so:
/// ```python
/// def func(foo):
///     return foo
/// ```
/// 
/// In Rusted Flask Jinja, arguments are passed as a Vec. 
/// The equivalent in Python would be:
/// ```python
/// def func(*args):
///     return args[0]
/// ```
/// 
/// # Examples
/// ```
/// fn func(arguments: Vec<String>) -> String {
///     arguments[0_usize].clone()
/// }
/// ```
pub type JinjaFunction = fn(Vec<String>) -> String;

/// An error from within Jinja.
///
/// This should be raised as an issue
#[derive(Debug)]
pub enum InternalJinjaError {
    /// A parser regex couldn't be read
    CantReadRegex(regex::Error),
}

/// An error with Jinja
///
/// This can come from your own code,
/// or from Jinja itself (see `InternalJinjaError`)
#[derive(Debug)]
pub enum JinjaError {
    /// An error from within Jinja
    /// See the `InternalJinjaError` enum
    InternalJinjaError(InternalJinjaError),
    /// The template could not be found
    TemplateNotFound,
    /// There was no such variable passed to Jinja
    NoSuchVariable,
    /// An other error occured
    Other(String),
}

/// Renders a template from a given string
pub fn render_template_string(
    template: String,
    variables: HashMap<&str, String>,
    functions: Option<HashMap<&str, JinjaFunction>>
) -> Result<String, JinjaError> {
    let mut rendered = String::new();
    let simple_variable = match Regex::new(consts::REPLACE) {
        Err(why) => {
            return Err(JinjaError::InternalJinjaError(
                InternalJinjaError::CantReadRegex(why),
            ))
        }
        Ok(regex) => regex,
    };
    for entry in simple_variable.captures_iter(&template) {
        let variable = &entry;
        let variable_value = match variables.get(&variable["variable"]) {
            None => return Err(JinjaError::NoSuchVariable),
            Some(val) => val,
        };
        rendered = template.replace(&variable[0], variable_value);
    }
    Ok(rendered)
}

/// Renders a template from a given file
pub fn render_template(file: &str, variables: HashMap<&str, String>, functions: Option<HashMap<&str, JinjaFunction>>) -> Result<String, JinjaError> {
    // Variables are <&str, String> because the key is more likely to be
    // a string const, and the value is more likely to be dynamically generated
    let fpath = Path::new("./templates/").join(file);
    let mut opened_file = match File::open(fpath) {
        Err(why) => {
            return Err(JinjaError::Other(format!(
                "can't open file, error: {}",
                why
            )))
        }
        Ok(file) => file,
    };

    let mut contents = String::new();

    match opened_file.read_to_string(&mut contents) {
        Err(why) => {
            return Err(JinjaError::Other(format!(
                "couldn't read file, error: {}",
                why
            )))
        }
        Ok(_) => return render_template_string(contents, variables, functions),
    }
}
