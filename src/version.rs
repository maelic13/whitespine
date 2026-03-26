pub fn display_version() -> &'static str {
    if is_release_build() {
        env!("CARGO_PKG_VERSION")
    } else {
        concat!(env!("CARGO_PKG_VERSION"), "-dev")
    }
}

fn is_release_build() -> bool {
    matches!(
        option_env!("WHITESPINE_RELEASE"),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}
