use serde::Deserialize;

use crate::download::{APPLICATION_GZIP, APPLICATION_OCTET_STREAM, APPLICATION_XZ};

/// The compressed archive of the compatibility tool and file name.
///
/// For GE Proton the archive is provided as a `tar.gz` file.<br>
/// For Wine GE the archive is provide as a `tar.xz` file.
pub struct DownloadedArchive {
    pub compressed_content: Vec<u8>,
    pub file_name: String,
}

impl DownloadedArchive {
    pub fn new(compressed_content: Vec<u8>, file_name: String) -> Self {
        DownloadedArchive {
            compressed_content,
            file_name,
        }
    }
}

/// The expected checksum of a compatibility tool and the checksum file name.
///
/// The checksum is provided as a `sha512sum` file.
pub struct DownloadedChecksum {
    pub checksum: String,
    pub file_name: String,
}

impl DownloadedChecksum {
    pub fn new(checksum: String, file_name: String) -> Self {
        DownloadedChecksum { checksum, file_name }
    }
}

/// Assets of a GE Proton or Wine GE release.
pub struct DownloadedAssets {
    /// Tag name of the release.
    pub tag: String,
    /// The compressed archive.
    pub compressed_archive: DownloadedArchive,
    /// The checksum of the compressed archive.
    ///
    /// The `checksum` for a archive can be `None` if `download_checksum` in `DownloadRequest` is
    /// set to false.
    pub checksum: Option<DownloadedChecksum>,
}

impl DownloadedAssets {
    pub fn new(tag: String, compressed_archive: DownloadedArchive, checksum: Option<DownloadedChecksum>) -> Self {
        DownloadedAssets {
            tag,
            compressed_archive,
            checksum,
        }
    }
}

/// Represents a GitHub API release.
///
/// Only the `tag_name` and `assets` of the release are relevant for us. Too see the APIs from which this struct is
/// constructed from see the documentation of `GeDownloader::fetch_release`.
#[derive(Debug, Deserialize)]
pub struct GeRelease {
    pub tag_name: String,
    pub assets: Vec<GeAsset>,
}

impl GeRelease {
    pub fn new(tag_name: String, assets: Vec<GeAsset>) -> Self {
        GeRelease { tag_name, assets }
    }

    fn is_checksum_asset(asset: &GeAsset) -> bool {
        asset.content_type.eq(APPLICATION_OCTET_STREAM)
    }

    fn is_tar_asset(asset: &GeAsset) -> bool {
        asset.content_type.eq(APPLICATION_GZIP) || asset.content_type.eq(APPLICATION_XZ)
    }

    pub fn checksum_asset(&self) -> &GeAsset {
        self.assets
            .iter()
            .find(|asset| GeRelease::is_checksum_asset(asset))
            .unwrap()
    }

    pub fn tar_asset(&self) -> &GeAsset {
        self.assets.iter().find(|asset| GeRelease::is_tar_asset(asset)).unwrap()
    }
}

/// An asset of a GitHub release.
///
/// This struct contains the URL from which the asset file can be downloaded from. Additionally, it contains the
/// content type of the file and the file name.
#[derive(Debug, Deserialize)]
pub struct GeAsset {
    /// File name of the asset.
    pub name: String,
    pub content_type: String,
    pub browser_download_url: String,
}

impl GeAsset {
    pub fn new<S: Into<String>>(name: S, content_type: S, browser_download_url: S) -> Self {
        GeAsset {
            name: name.into(),
            content_type: content_type.into(),
            browser_download_url: browser_download_url.into(),
        }
    }
}

impl Clone for GeAsset {
    fn clone(&self) -> Self {
        GeAsset {
            name: self.name.clone(),
            content_type: self.content_type.clone(),
            browser_download_url: self.content_type.clone(),
        }
    }
}

/// Newtype for GitHub API tag name deserialization.
#[derive(Debug, Deserialize)]
pub(crate) struct CompatibilityToolTag {
    name: String,
}

impl From<CompatibilityToolTag> for String {
    fn from(tag_name: CompatibilityToolTag) -> Self {
        String::from(tag_name.name)
    }
}

#[cfg(test)]
mod ge_release_tests {
    use crate::download::{APPLICATION_GZIP, APPLICATION_OCTET_STREAM};

    use super::*;

    #[test]
    fn get_checksum_asset() {
        let tag = String::from("6.20-GE-1");
        let assets = vec![
            GeAsset::new("Proton-6.20-GE-1.tar.gz", APPLICATION_GZIP, "gzip"),
            GeAsset::new("Proton-6.20-GE-1.sha512sum", APPLICATION_OCTET_STREAM, "octet"),
        ];
        let release = GeRelease::new(tag, assets);

        let checksum_asset = release.checksum_asset();
        assert_eq!(checksum_asset.name, "Proton-6.20-GE-1.sha512sum");
        assert_eq!(checksum_asset.content_type, APPLICATION_OCTET_STREAM);
        assert_eq!(checksum_asset.browser_download_url, "octet");
    }

    #[test]
    fn get_archive_asset() {
        let tag = String::from("6.20-GE-1");
        let assets = vec![
            GeAsset::new("Proton-6.20-GE-1.tar.gz", APPLICATION_GZIP, "gzip"),
            GeAsset::new("Proton-6.20-GE-1.sha512sum", APPLICATION_OCTET_STREAM, "octet"),
        ];
        let release = GeRelease::new(tag, assets);

        let gzip_asset = release.tar_asset();
        assert_eq!(gzip_asset.name, "Proton-6.20-GE-1.tar.gz");
        assert_eq!(gzip_asset.content_type, APPLICATION_GZIP);
        assert_eq!(gzip_asset.browser_download_url, "gzip");
    }
}
