pub const REPLACE: &str = r#"\{\{ (?P<variable>.*) \}\}"#;

pub const INCLUDE: &str = r#"\{% include "(?P<filename>.*)" %\}"#;
