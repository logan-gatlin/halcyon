use std::path::{
    Path,
    PathBuf,
};

use halcyon_lib::tooling::TextPosition;
use lsp_types::{
    Position,
    Range,
    Uri,
};

pub fn text_range(
    start: TextPosition,
    end: TextPosition,
) -> Range {
    Range {
        start: Position {
            line: start.line,
            character: start.character,
        },
        end: Position {
            line: end.line,
            character: end.character,
        },
    }
}

pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let url = url::Url::parse(uri.as_str()).ok()?;
    url.to_file_path().ok().map(|path| normalize_path(&path))
}

pub fn path_to_uri(path: &Path) -> Option<Uri> {
    url::Url::from_file_path(path)
        .ok()?
        .as_str()
        .parse::<Uri>()
        .ok()
}

pub fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
