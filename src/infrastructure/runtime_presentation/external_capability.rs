pub(super) fn headline(detail: &str) -> String {
    let first_line = detail.lines().next().unwrap_or_default().trim();
    if first_line.is_empty() {
        "external capability".to_string()
    } else {
        first_line.to_string()
    }
}

pub(super) fn badge_class(detail: &str) -> &'static str {
    match status(detail) {
        Some("succeeded") | None => "tool",
        Some(_) => "fallback",
    }
}

fn status(detail: &str) -> Option<&str> {
    detail.lines().next().and_then(|line| {
        line.split_whitespace()
            .find_map(|segment| segment.strip_prefix("status="))
    })
}
