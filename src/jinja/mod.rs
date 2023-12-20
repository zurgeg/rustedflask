mod consts;

use regex::Regex;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::Read,
    path::Path,
};

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
pub enum JinjaError<'a> {
    /// An error from within Jinja
    /// See the `InternalJinjaError` enum
    InternalJinjaError(InternalJinjaError),
    /// The template could not be found
    TemplateNotFound,
    /// There was no such variable passed to Jinja
    NoSuchVariable,
    /// There was no such function passed to Jinja
    NoSuchFunction,
    /// Syntax was invalid
    SyntaxError(&'a str),
    /// An other error occured
    Other(String),
}
fn parse_replace<'a>(
    varname: &str,
    variables: &HashMap<&'a str, String>,
    functions: Option<HashMap<&'a str, JinjaFunction>>,
) -> Result<(bool, String, Vec<String>, VecDeque<u8>), JinjaError<'a>> {
    loop {
        let mut is_function = false;
        let mut function_name = String::new();
        let mut function_args = Vec::<String>::new();
        let mut varname_chars = VecDeque::from(varname.to_string().into_bytes());
        let curchar = match varname_chars.pop_front() {
            None => break,
            Some(val) => val,
        };
        if curchar == b'(' {
            if function_name == "".to_string() {
                return Err(JinjaError::SyntaxError("Function call with no name"));
            }
        }
    }
    unreachable!();
}

/// Renders a template from a given string
pub fn render_template_string<'a>(
    template: String,
    variables: HashMap<&'a str, String>,
    functions: Option<HashMap<&'a str, JinjaFunction>>,
) -> Result<String, JinjaError<'a>> {
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
        let varname = &variable["variable"];
        let mut is_function = false;
        let mut function_name = String::new();
        let mut function_args = Vec::<String>::new();
        let mut varname_chars = VecDeque::from(varname.to_string().into_bytes());

        let variable_value = match variables.get(&varname) {
            None => return Err(JinjaError::NoSuchVariable),
            Some(val) => val,
        };
        rendered = template.replace(&variable[0], variable_value);
    }

    Ok(rendered)
}

/// Renders a template from a given file
pub fn render_template<'a>(
    file: &'a str,
    variables: HashMap<&'a str, String>,
    functions: Option<HashMap<&'a str, JinjaFunction>>,
) -> Result<String, JinjaError<'a>> {
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
