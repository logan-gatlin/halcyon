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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{
        SystemTime,
        UNIX_EPOCH,
    };

    fn unique_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn path_and_uri_round_trip_for_existing_file() {
        let root = unique_path("halcyon-lsp-util");
        std::fs::create_dir_all(&root).expect("temp directory should be created");
        let file_path = root.join("bundle.hc");
        std::fs::write(&file_path, "bundle demo\n").expect("temp source file should be written");

        let uri = path_to_uri(&file_path).expect("file path should convert to URI");
        let round_trip = uri_to_path(&uri).expect("URI should convert back to file path");
        assert_eq!(round_trip, normalize_path(&file_path));

        std::fs::remove_dir_all(&root).expect("temp directory should be removed");
    }

    #[test]
    fn uri_to_path_rejects_non_file_schemes() {
        let uri = "https://example.com/demo.hc"
            .parse::<Uri>()
            .expect("test URI should parse");
        assert!(uri_to_path(&uri).is_none());
    }

    #[test]
    fn normalize_path_returns_original_for_missing_paths() {
        let missing_path = unique_path("halcyon-lsp-missing").join("bundle.hc");
        assert_eq!(normalize_path(&missing_path), missing_path);
    }
}
