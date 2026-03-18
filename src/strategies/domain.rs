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
