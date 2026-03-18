pub fn strategy_domain(
    source: &str,
    target: &str,
    haystack: &str,
) -> anyhow::Result<Option<String>> {
    let Ok(mut url) = url::Url::parse(haystack) else {
        // Swallow url parse error, dont care ab input
        return Ok(None);
    };

    let given_host = url.host_str();

    let given_host = match given_host {
        None => {
            tracing::warn!(source=%source, target=%target, url=%url, "Failed to extract host from current URL");
            return Ok(None);
        }
        Some(host) if host != source => {
            tracing::trace!(source=%source, target=%target, url=%url, host=%host, "Skipping non-matching URL");
            return Ok(None);
        }
        Some(host) => host.to_string(),
    };

    url.set_host(Some(target))
        .inspect_err(|e| {
        tracing::error!(source=%source, target=%target, error=%e, error_context=?e, url=%url, host=%target, "Failed to set host: {e}");
    })
        .inspect(|_| {
        tracing::debug!(source=%source, target=%target, original_host=%given_host, new_host=%url.host_str().unwrap(), original_content=%haystack, new_content=%url, "Applied domain");
    })?;

    Ok(Some(url.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- successful replacement ---

    #[test]
    fn replaces_matching_domain() {
        let result = strategy_domain("x.com", "example.com", "https://x.com/foo/bar").unwrap();
        assert_eq!(result, Some("https://example.com/foo/bar".to_string()));
    }

    #[test]
    fn preserves_path_query_and_fragment() {
        let input = "https://x.com/path?q=1&r=2#section";
        let result = strategy_domain("x.com", "example.com", input).unwrap();
        assert_eq!(result, Some("https://example.com/path?q=1&r=2#section".to_string()));
    }

    // --- real-world: x.com → stupidpenisx.com (from default config) ---

    #[test]
    fn redirects_x_com_to_target() {
        let result =
            strategy_domain("x.com", "stupidpenisx.com", "https://x.com/user/status/123")
                .unwrap();
        assert_eq!(result, Some("https://stupidpenisx.com/user/status/123".to_string()));
    }

    // --- non-matching domain ---

    #[test]
    fn returns_none_for_different_domain() {
        let result = strategy_domain("x.com", "example.com", "https://twitter.com/foo").unwrap();
        assert_eq!(result, None);
    }

    // --- invalid / non-URL input ---

    #[test]
    fn returns_none_for_plain_text() {
        let result = strategy_domain("x.com", "example.com", "just some plain text").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_empty_string() {
        let result = strategy_domain("x.com", "example.com", "").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_relative_path() {
        let result = strategy_domain("x.com", "example.com", "/relative/path").unwrap();
        assert_eq!(result, None);
    }

    // --- scheme variations ---

    #[test]
    fn works_with_http_scheme() {
        let result = strategy_domain("x.com", "example.com", "http://x.com/page").unwrap();
        assert_eq!(result, Some("http://example.com/page".to_string()));
    }

    // --- host == None branch (warn path) ---

    #[test]
    fn returns_none_when_url_has_no_host() {
        // file:///path parses successfully but host_str() is None
        let result = strategy_domain("x.com", "example.com", "file:///local/path").unwrap();
        assert_eq!(result, None);
    }

    // --- set_host error path ---

    #[test]
    fn returns_error_when_target_host_is_invalid() {
        // An empty host on a non-file URL causes set_host to return Err
        let result = strategy_domain("x.com", "", "https://x.com/page");
        assert!(result.is_err());
    }
}
