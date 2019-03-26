// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Utilities for binding generators.

use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// Recursively copy all files with the given extension from the source to the target directories.
pub fn copy_files<S: AsRef<Path>, T: AsRef<Path>>(
    source: S,
    target: T,
    extension: &str,
) -> io::Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    for entry in WalkDir::new(source) {
        let entry = entry?;

        if entry.path().is_file()
            && entry
                .path()
                .to_str()
                .map(|s| s.ends_with(extension))
                .unwrap_or(false)
        {
            let source_path = entry.path();
            let target_path = target.join(source_path.strip_prefix(source).unwrap_or(source_path));

            let _ = fs::copy(source_path, target_path)?;
        }
    }

    Ok(())
}
