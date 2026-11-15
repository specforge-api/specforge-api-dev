//! Static asset embedding at compile time for zero-dependency deployment.

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontend"]
pub struct Asset;

impl Asset {
    pub fn get_index_html() -> Option<String> {
        let file = Asset::get("index.html")?;
        std::str::from_utf8(file.data.as_ref()).ok().map(|s| s.to_string())
    }
}

