use std::collections::HashMap;

pub mod pages;

use crate::config::Config;

pub(crate) fn create_path_map(_config: impl AsRef<Config>) -> HashMap<&'static str, String> {
    const STATIC_ASSETS: [&'static str; 11] = [
        "addfiles.svg",
        "app.css",
        "app.js",
        "apple-touch-icon.png",
        "favicon-16x16.png",
        "favicon-32x32.png",
        "icon.svg",
        "safari-pinned-tab.svg",
        "send-fb.jpg",
        "send-twitter.jpg",
        "wordmark.svg",
    ];

    let mut res = HashMap::new();

    // Add static assets:
    for resource in STATIC_ASSETS {
        res.insert(resource, format!("/static/{resource}"));
    }

    res
}
