//! JSON Pointer paths, as RFC 6901 writes them.
//!
//! The credential check and the comparison both name a place inside a JSON
//! value, and both escape a key the same way: `~` becomes `~0` and `/` becomes
//! `~1`, so the key `a/b` reads as `/a~1b` and `a~b` reads as `/a~0b`.
//!
//! <https://www.rfc-editor.org/rfc/rfc6901>

// Groups 5 through 10 of the export-import change call these from the prompt
// and assistant commands. Until those land, the unit tests are the only callers.
#![allow(dead_code)]

/// Escape one key for a pointer: `~` first, then `/`, as RFC 6901 section 4 asks.
pub fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// Read one escaped pointer token back as the key it names.
pub fn unescape(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// Add one key to a pointer.
pub fn child(path: &str, token: &str) -> String {
    format!("{}/{}", path, escape(token))
}

/// Add one array index to a pointer.
pub fn index(path: &str, position: usize) -> String {
    format!("{}/{}", path, position)
}

/// The key a value at `path` sits under, with array indices stepped over.
///
/// A string inside an array takes the key of the array, so
/// `/assistant/config/api_keys/0` answers `api_keys`. The root pointer, the
/// empty string, answers `None`.
pub fn key_of(path: &str) -> Option<String> {
    path.rsplit('/')
        .find(|token| !token.is_empty() && !token.chars().all(|c| c.is_ascii_digit()))
        .map(unescape)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_a_slash_and_a_tilde() {
        assert_eq!(child("", "a/b"), "/a~1b");
        assert_eq!(child("", "a~b"), "/a~0b");
        assert_eq!(child("", "a~/b"), "/a~0~1b");
    }

    #[test]
    fn reads_an_escaped_token_back() {
        assert_eq!(unescape("a~1b"), "a/b");
        assert_eq!(unescape("a~0b"), "a~b");
        assert_eq!(unescape("a~01"), "a~1");
    }

    #[test]
    fn names_the_key_a_value_sits_under() {
        assert_eq!(
            key_of("/assistant/config/configurable/openai_api_key").as_deref(),
            Some("openai_api_key")
        );
        assert_eq!(
            key_of("/assistant/config/api_keys/0").as_deref(),
            Some("api_keys")
        );
        assert_eq!(key_of("/a~1b").as_deref(), Some("a/b"));
        assert_eq!(key_of(""), None);
    }
}
