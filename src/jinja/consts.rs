pub const REPLACE: &str = r#"\{\{ (?P<variable>.*) \}\}"#;

pub const INCLUDE: &str = r#"\{% include "(?P<filename>.*)" %\}"#;

pub const EXTEND: &str = r#"\{% extend "(?P<filename>.*)" %\}"#;

pub const BLOCK_START: &str = r"\{% block (?P<blockname>.*) %\}";

pub const BLOCK_END: &str = r#"\{% endblock %\}"#;