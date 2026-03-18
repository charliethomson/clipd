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
