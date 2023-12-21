pub const REPLACE: &str = r#"\{\{ (?P<variable>.*) \}\}"#;

pub const INCLUDE: &str = r#"\{% include "(?P<filename>.*)" %\}"#;

pub const EXTEND: &str = r#"\{% extends "(?P<filename>.*)" %\}"#;

pub const BLOCK: &str = r"(?ms)\{% block (?P<blockname>.*) %\}\n?(?P<blockcontent>.*)\n?\{% endblock %\}";