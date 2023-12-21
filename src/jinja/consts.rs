use lazy_static::lazy_static;

macro_rules! load_regex {
    ($name:ident, $regex:expr) => {
        lazy_static! {
            pub static ref $name: regex::Regex = match regex::Regex::new($regex) {
                Err(why) => {
                    panic!("{}", why)
                }
                Ok(regex) => regex,
            };
        }
    }
}

load_regex!(REPLACE, r#"\{\{ (?P<variable>.*) \}\}"#);

load_regex!(INCLUDE, r#"\{% include "(?P<filename>.*)" %\}"#);

load_regex!(EXTEND, r#"\{% extends "(?P<filename>.*)" %\}(?P<strip>(.|\n)*)"#);

load_regex!(BLOCK, r"(?ms)\{% block (?P<blockname>.*) %\}\n?(?P<blockcontent>.*)\n?\{% endblock %\}");
