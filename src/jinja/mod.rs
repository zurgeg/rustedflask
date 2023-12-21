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
pub enum JinjaError {
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
    SyntaxError(String),
    /// The template could not be opened
    NoSuchTemplate,
    /// There were more than two parents in the template
    MultipleParentsError,
    /// An other error occured
    Other(String),
}
fn parse_replace<'a>(
    varname: &str,
    variables: &HashMap<&'a str, String>,
) -> Result<(bool, String, Vec<String>), JinjaError> {
    let mut is_function = false;
    let mut function_name = String::new();
    let mut function_args = Vec::<String>::new();
    let mut varname_chars = VecDeque::from(varname.to_string().into_bytes());
    loop {
        let curchar = match varname_chars.pop_front() {
            None => break,
            Some(val) => val,
        };
        if curchar == b'(' {
            is_function = true;
            if function_name == "".to_string() {
                return Err(JinjaError::SyntaxError("Function call with no name".into()));
            } else {
                // Start parsing arguments
                loop {
                    let curchar = match varname_chars.pop_front() {
                        None => return Err(JinjaError::SyntaxError("Unclosed parentheses".into())),
                        Some(val) => val,
                    };
                    if curchar == b'"' {
                        let mut string_lit = String::new();
                        // Start parsing a string literal
                        loop {
                            let curchar = match varname_chars.pop_front() {
                                None => {
                                    return Err(JinjaError::SyntaxError(
                                        "Unclosed string literal".into(),
                                    ))
                                }
                                Some(val) => val,
                            };
                            if curchar == b'"' {
                                function_args.push(string_lit.clone());
                                let curchar = match varname_chars.pop_front() {
                                    None => {
                                        return Err(JinjaError::SyntaxError(
                                            "Unclosed parentheses".into(),
                                        ))
                                    }
                                    Some(val) => val,
                                };
                                match curchar {
                                    b',' => {
                                        varname_chars.push_back(b',');
                                        break;
                                    }
                                    b')' => return Ok((is_function, function_name, function_args)),
                                    somethingelse => {
                                        return Err(JinjaError::SyntaxError(format!(
                                            "Expected comma or closing parentheses, got \"{}\"",
                                            char::from(somethingelse)
                                        ))
                                        .into())
                                    }
                                }
                            }
                            string_lit.push(curchar.into());
                        }
                    } else if curchar == b')' {
                        return Ok((is_function, function_name, function_args));
                    } else if curchar == b',' || curchar == b' ' {
                        continue;
                    } else {
                        // it's a variable, start parsing
                        let mut varname = String::new();
                        varname.push(curchar.into());
                        let mut curchar: u8;
                        loop {
                            curchar = match varname_chars.pop_front() {
                                None => {
                                    return Err(JinjaError::SyntaxError(
                                        "Unclosed parentheses".into(),
                                    ))
                                }
                                Some(val) => val,
                            };
                            if curchar == b',' || curchar == b')' {
                                break;
                            }
                            if curchar == b' ' {
                                return Err(JinjaError::SyntaxError(
                                    "Expected a variable name, but a space was found".into(),
                                ));
                            } else {
                                varname.push(curchar.into());
                            }
                        }
                        let varval = match variables.get(&*varname) {
                            None => return Err(JinjaError::NoSuchVariable),
                            Some(val) => val,
                        };
                        function_args.push(varval.clone());
                        if curchar == b')' {
                            return Ok((is_function, function_name, function_args));
                        }
                    }
                }
            }
        }
        function_name.push(curchar.into());
    }
    if !is_function {
        return Ok((is_function, String::new(), vec![]));
    };
    unreachable!()
}

/// Renders a template from a given string
pub fn render_template_string<'a>(
    template: String,
    variables: HashMap<&'a str, String>,
    functions: Option<HashMap<&'a str, JinjaFunction>>,
) -> Result<String, JinjaError> {
    let mut rendered = template.clone();
    let simple_variable = match Regex::new(consts::REPLACE) {
        Err(why) => {
            return Err(JinjaError::InternalJinjaError(
                InternalJinjaError::CantReadRegex(why),
            ))
        }
        Ok(regex) => regex,
    };
    let inclusion = match Regex::new(consts::INCLUDE) {
        Err(why) => {
            return Err(JinjaError::InternalJinjaError(
                InternalJinjaError::CantReadRegex(why),
            ))
        }
        Ok(regex) => regex,
    };

    let extend = match Regex::new(consts::EXTEND) {
        Err(why) => {
            return Err(JinjaError::InternalJinjaError(
                InternalJinjaError::CantReadRegex(why),
            ))
        }
        Ok(regex) => regex,
    };

    let block = match Regex::new(consts::BLOCK) {
        Err(why) => {
            return Err(JinjaError::InternalJinjaError(
                InternalJinjaError::CantReadRegex(why),
            ))
        }
        Ok(regex) => regex,
    };

    let temp_render_clone = rendered.clone();
    let extends = extend.captures(&temp_render_clone);

    if let Some(parents) = extends {
        let filename = Path::new("./templates/").join(Path::new(&parents["filename"]));
        let mut file = match File::open(filename) {
            Err(_) => return Err(JinjaError::NoSuchTemplate),
            Ok(file) => file,
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(_) => return Err(JinjaError::Other("Could not read template file".into())),
            Ok(_) => {}
        };
        {
            let temp_contents_clone = contents.clone();
            let parent_blocks = block.captures_iter(&*temp_contents_clone);
            let child_blocks = block.captures_iter(&*temp_render_clone);
            let mut child_map = HashMap::new();
            for block in child_blocks {
                child_map.insert(block["blockname"].to_string(), block["blockcontent"].to_string());
            };
            for block in parent_blocks {
                if let Some(child_block) = child_map.get(&block["blockname"].to_string()) {
                    contents = temp_contents_clone.replace(&block[0], &*child_block)
                }
            }
        }
        rendered = temp_render_clone.replace(&parents[0], &*contents).replace(&parents["strip"], "");

    }

    for entry in inclusion.captures_iter(&rendered.clone()) {
        let filename = Path::new("./templates/").join(Path::new(&entry["filename"]));
        let mut file = match File::open(filename) {
            Err(_) => return Err(JinjaError::NoSuchTemplate),
            Ok(file) => file,
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(_) => return Err(JinjaError::Other("Could not read template file".into())),
            Ok(_) => {}
        };
        rendered = rendered.replace(&entry[0], &*contents);
    }

    for entry in simple_variable.captures_iter(&rendered.clone()) {
        let variable = &entry;
        let varname = &variable["variable"];

        let (is_function, function_name, function_args) = match parse_replace(varname, &variables) {
            Err(why) => return Err(why),
            Ok(value) => value,
        };
        if is_function {
            match functions {
                Some(ref functions) => {
                    let functions = functions.clone();
                    let function = match functions.get(&*function_name) {
                        Some(function) => function,
                        None => return Err(JinjaError::NoSuchFunction),
                    };
                    let value = function(function_args);
                    rendered = rendered.replace(&variable[0], &*value);
                }
                None => return Err(JinjaError::NoSuchFunction),
            }
        } else {
            let variable_value = match variables.get(&varname) {
                None => return Err(JinjaError::NoSuchVariable),
                Some(val) => val,
            };
            rendered = rendered.replace(&variable[0], variable_value);
        };
        return Ok(rendered);
    }

    Ok(rendered)
}

/// Renders a template from a given file
pub fn render_template<'a>(
    file: &'a str,
    variables: HashMap<&'a str, String>,
    functions: Option<HashMap<&'a str, JinjaFunction>>,
) -> Result<String, JinjaError> {
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
