use regex::Regex;

pub fn strategy_regex(
    pattern: &str,
    replacement: &str,
    haystack: &str,
) -> anyhow::Result<Option<String>> {
    let re = Regex::new(pattern)?;

    let new_content = re.replace(haystack, replacement).to_string();

    if new_content != haystack {
        tracing::debug!(pattern=%pattern, replacement=%replacement, original_content=%haystack, new_content=%new_content, "Applied match");
        Ok(Some(new_content))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- match / replacement ---

    #[test]
    fn replaces_matching_pattern() {
        let result = strategy_regex(r"foo", "bar", "foo baz").unwrap();
        assert_eq!(result, Some("bar baz".to_string()));
    }

    #[test]
    fn returns_none_when_no_match() {
        let result = strategy_regex(r"foo", "bar", "qux quux").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_when_replacement_produces_same_string() {
        // Pattern matches but the replacement text is identical to what was there.
        let result = strategy_regex(r"foo", "foo", "foo bar").unwrap();
        assert_eq!(result, None);
    }

    // --- capture groups ---

    #[test]
    fn capture_group_replacement() {
        let result = strategy_regex(r"(\d+)-(\d+)", "$2-$1", "2024-03").unwrap();
        assert_eq!(result, Some("03-2024".to_string()));
    }

    // --- real-world: YouTube URL normalisation (from default config) ---

    #[test]
    fn normalises_youtube_url() {
        let pattern = r"^(.+)v=([a-zA-Z0-9-_]{11})(.+)$";
        let replacement = "https://youtube.com/watch?v=$2";
        let input = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=related";
        let result = strategy_regex(pattern, replacement, input).unwrap();
        assert_eq!(result, Some("https://youtube.com/watch?v=dQw4w9WgXcQ".to_string()));
    }

    #[test]
    fn youtube_pattern_no_match_for_non_youtube_url() {
        let pattern = r"^(.+)v=([a-zA-Z0-9-_]{11})(.+)$";
        let replacement = "https://youtube.com/watch?v=$2";
        let input = "https://example.com/page";
        let result = strategy_regex(pattern, replacement, input).unwrap();
        assert_eq!(result, None);
    }

    // --- only first occurrence is replaced (regex::Regex::replace behaviour) ---

    #[test]
    fn replaces_only_first_occurrence() {
        let result = strategy_regex(r"x", "y", "x and x").unwrap();
        assert_eq!(result, Some("y and x".to_string()));
    }

    // --- error: invalid pattern ---

    #[test]
    fn invalid_pattern_returns_error() {
        let result = strategy_regex(r"[unclosed", "bar", "anything");
        assert!(result.is_err());
    }
}
