//! Possible errors that can be thrown by this crate.
use std::io;

use reqwest::blocking::Response;
use thiserror::Error;

use crate::tag::TagKind;

/// Error for Steam config related problems.
#[derive(Error, Debug)]
pub enum SteamConfigError {
    /// The Steam config contains no `CompatToolMapping` group/array.
    #[error("Config to copy has no CompatToolMapping group")]
    NoDefaultCompatToolAttribute,
    /// An IO error occurred while working with the Steam config.
    #[error("IO error occurred - Inspect the source for more information")]
    IoError {
        #[from]
        source: io::Error,
    },
}

/// Error for Lutris config related errors.
#[derive(Error, Debug)]
pub enum LutrisConfigError {
    /// The Lutris config contains no `version` attribute.
    #[error("Config to copy has no version attribute")]
    NoVersionAttribute,
    /// An IO error occurred while working with the Lutris config.
    #[error("IO error occurred - Inspect the source for more information")]
    IoError {
        #[from]
        source: io::Error,
    },
}

/// Errors for `serde` related issues.
#[derive(Error, Debug)]
pub enum DeserializeError {
    /// Could not convert the JSON response of the GitHub API to a struct.
    #[error("Could not convert Github JSON response into a struct")]
    FailedToConvertToStruct {
        #[from]
        source: Box<dyn std::error::Error>,
    },
}

/// Errors for GitHub API or `reqwest` related errors.
#[derive(Debug, Error)]
pub enum GithubError {
    /// GitHub API response could not be converted with the `serde` crate.
    #[error("Failed to convert GitHub resource with serde")]
    SerdeDeserializeError {
        #[from]
        source: serde_json::Error,
    },
    /// Reqwest could not fetch a resource from the GitHub API.
    #[error("Failed to fetch resource from GitHub API")]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    /// The GitHub API returned no release tags.
    #[error("No tags could be found")]
    NoTags,
    /// The GitHub API returned no assets for the fetched release.
    #[error("For {tag} {kind} the release has no assets")]
    ReleaseHasNoAssets { tag: String, kind: TagKind },
    /// The response of the GitHub API is not HTTP code 200 (OK).
    #[error("HTTP response status was not OK (200)")]
    StatusNotOk(Response),
}

/// Error for when a `TagKind` can not be created.
#[derive(Debug, Error)]
pub enum TagKindError {
    /// A `TagKind` could not be created from the provide string.
    #[error("Could not create TagKind from provided string.")]
    UnknownString,
}
