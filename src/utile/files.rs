use std::{
    io,
    path::{Path, PathBuf},
};

use crate::{http::Error::HTTPError, http_err};



pub fn is_in_public(_path: &str) -> Result<Option<PathBuf>, HTTPError> {
    let path = _path.trim_start_matches('/');

    let base = Path::new("public")
        .canonicalize()
        .map_err(|e| http_err!(InternalServerError, "public folder unavailable: {}", e))?;

    let full = match base.join(path).canonicalize() {
        Ok(p) => p,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(http_err!(
                InternalServerError,
                "cannot resolve {}: {}",
                path,
                e
            ));
        }
    };

    if full.starts_with(&base) {
        Ok(Some(full))
    } else {
        Ok(None)
    }
}
