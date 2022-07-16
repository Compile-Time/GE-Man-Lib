//! Structs for representing a GitHub release tag.
//!
//! This module provides structs for working with GitHub release tags.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::Path;

use lazy_static::lazy_static;
use regex::{Captures, Match, Regex};
use serde::{Deserialize, Serialize};

use crate::error::TagKindError;

const PROTON: &str = "PROTON";
const WINE: &str = "WINE";
const LOL_WINE: &str = "LOL_WINE";

const RELEASE_CANDIDATE_MARKER: &str = "rc";
const FIRST_GROUP: usize = 1;

lazy_static! {
    static ref NUMBERS: Regex = Regex::new(r"(\d+)").unwrap();
    static ref TAG_MARKERS: Vec<String> = vec![String::from("rc"), String::from("LoL"), String::from("MF")];
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SemVer {
    major: u8,
    minor: u8,
    patch: u8,
    identifier: Option<String>,
}

impl SemVer {
    fn new(major: u8, minor: u8, patch: u8, identifier: Option<String>) -> Self {
        SemVer {
            major,
            minor,
            patch,
            identifier,
        }
    }

    pub fn identifier(&self) -> &Option<String> {
        &self.identifier
    }

    pub fn major(&self) -> u8 {
        self.major
    }

    pub fn minor(&self) -> u8 {
        self.minor
    }

    pub fn patch(&self) -> u8 {
        self.patch
    }

    pub fn str(&self) -> String {
        if self.identifier.is_some() {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.identifier.as_ref().unwrap())
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    fn from_git_tag(git_tag: &String) -> Self {
        let number_captures: Vec<Captures> = NUMBERS.captures_iter(&git_tag).collect();

        let semver = if git_tag.contains(RELEASE_CANDIDATE_MARKER) {
            if let Some(rc_match) = SemVer::get_rc_match(&git_tag, &number_captures) {
                let captures_without_rc: Vec<Captures> = number_captures
                    .into_iter()
                    .filter(|cap| cap.get(FIRST_GROUP).unwrap().ne(&rc_match))
                    .collect();
                let mut semver = SemVer::create_semver_from_regex(&captures_without_rc);
                let rc_marker = format!("rc{}", rc_match.as_str());

                semver.identifier = Some(rc_marker);
                semver
            } else {
                panic!("Git tag is not parsable!");
            }
        } else {
            let mut semver = SemVer::create_semver_from_regex(&number_captures);

            for marker in &*TAG_MARKERS {
                if git_tag.contains(marker) {
                    semver.identifier = Some(marker.to_owned());
                }
            }

            semver
        };

        semver
    }

    fn create_semver_from_regex(captures: &[Captures]) -> Self {
        let mut numbers: Vec<u8> = Vec::with_capacity(3);

        for cap in captures {
            numbers.push((&cap[1]).parse().unwrap())
        }

        // In the case that we do not have enough matches to fill the semver string we fill it with empty zeros.
        let numbers_len = numbers.len();
        if numbers_len < 3 {
            for _ in numbers_len..3 {
                numbers.push(0);
            }
        }

        SemVer::new(numbers[0], numbers[1], numbers[2], None)
    }

    fn get_rc_match<'a>(git_tag: &String, number_captures: &Vec<Captures<'a>>) -> Option<Match<'a>> {
        // Skip the first version number match because it might be the same number as the rc candidate.
        for cap in number_captures.iter().skip(1) {
            let version_number = &cap[FIRST_GROUP];
            let rc_query = format!("{}{}", RELEASE_CANDIDATE_MARKER, version_number);
            if git_tag.contains(&rc_query) {
                // Since every match contains a single capture group we always
                return Some(cap.get(FIRST_GROUP).unwrap().clone());
            }
        }
        None
    }
}

impl Display for SemVer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

/// Struct for representing a GitHub release tag and providing an option to be transformed into a semantic version.
///
/// Internally this struct parses the given release tag to create a semantic version representation from it. This is
/// done because it is much easier to perform comparisons between `6.20.1` and `7.8.0` than with `Proton-6.20-GE-1` and
/// `GE-Proton7-8`.
///
/// This struct supports `serde`'s serialization and deserialization traits.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tag {
    // Alias for versions before version 0.2.0.
    #[serde(alias = "value")]
    str: String,
    semver: SemVer,
}

impl Tag {
    pub fn new<S: Into<String>>(git_tag: S) -> Self {
        let value = git_tag.into();
        let semver = SemVer::from_git_tag(&value);

        Tag { str: value, semver }
    }

    /// Get this `Tag` as a semantic version.
    pub fn semver(&self) -> &SemVer {
        &self.semver
    }

    /// Get the string value of this `Tag`.
    pub fn str(&self) -> &String {
        &self.str
    }
}

impl Default for Tag {
    fn default() -> Self {
        Tag::new("")
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Tag::new(&s)
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Tag::new(s)
    }
}

impl From<Option<String>> for Tag {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(str) => Tag::new(str),
            None => Tag::default(),
        }
    }
}

impl From<Option<&str>> for Tag {
    fn from(opt: Option<&str>) -> Self {
        match opt {
            Some(str) => Tag::new(str),
            None => Tag::default(),
        }
    }
}

impl AsRef<Path> for Tag {
    fn as_ref(&self) -> &Path {
        self.str.as_ref()
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.str.as_ref()
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl PartialEq<Tag> for Tag {
    fn eq(&self, other: &Tag) -> bool {
        self.semver.eq(other.semver())
    }
}

impl PartialOrd<Tag> for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<Ordering> {
        self.semver.partial_cmp(other.semver())
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.semver.cmp(other.semver())
    }
}

impl Eq for Tag {}

impl From<Tag> for String {
    fn from(tag: Tag) -> Self {
        String::from(tag.str())
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.str.hash(state)
    }
}

/// Represents the kind of version for a `Tag`.
///
/// GE versions exists for both Proton and Wine. Additionally, for Wine also League of Legends specific versions
/// exist. Therefore, all possible version kinds are represented by this enum.
///
/// This enum supports `serde`'s serialization and deserialization traits.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
#[serde(tag = "type")]
pub enum TagKind {
    Proton,
    Wine { kind: WineTagKind },
}

impl TagKind {
    /// Create a Wine GE `TagKind`.
    pub fn wine() -> TagKind {
        TagKind::Wine {
            kind: WineTagKind::WineGe,
        }
    }

    /// Create a Wine GE LoL `TagKind`.
    pub fn lol() -> TagKind {
        TagKind::Wine {
            kind: WineTagKind::LolWineGe,
        }
    }

    /// Get all possible values.
    pub fn values() -> Vec<TagKind> {
        vec![TagKind::Proton, TagKind::wine(), TagKind::lol()]
    }

    /// Get a "human readable" compatibility tool name for the `TagKind`.
    pub fn compatibility_tool_name(&self) -> String {
        let name = match self {
            TagKind::Proton => "Proton GE",
            TagKind::Wine { kind } => match kind {
                WineTagKind::WineGe => "Wine GE",
                WineTagKind::LolWineGe => "Wine GE (LoL)",
            },
        };
        String::from(name)
    }

    /// Get a "human readable" compatibility tool kind text for the `TagKind`.
    pub fn compatibility_tool_kind(&self) -> String {
        let name = match self {
            TagKind::Proton => "Proton",
            TagKind::Wine { .. } => "Wine"
        };
        String::from(name)
    }

    /// Get a 1:1 string representation of the enum name.
    pub fn str(&self) -> String {
        let name = match self {
            TagKind::Proton => PROTON,
            TagKind::Wine { kind } => match kind {
                WineTagKind::WineGe => WINE,
                WineTagKind::LolWineGe => LOL_WINE,
            },
        };
        String::from(name)
    }

    fn from_str(str: &str) -> Result<Self, TagKindError> {
        let kind = match str {
            PROTON => TagKind::Proton,
            WINE => TagKind::wine(),
            LOL_WINE => TagKind::lol(),
            _ => return Err(TagKindError::UnknownString),
        };
        Ok(kind)
    }
}

impl From<&WineTagKind> for TagKind {
    fn from(kind: &WineTagKind) -> Self {
        TagKind::Wine { kind: *kind }
    }
}

impl From<WineTagKind> for TagKind {
    fn from(kind: WineTagKind) -> Self {
        TagKind::Wine { kind }
    }
}

impl TryFrom<&str> for TagKind {
    type Error = TagKindError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        TagKind::from_str(value)
    }
}

impl TryFrom<String> for TagKind {
    type Error = TagKindError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TagKind::from_str(&value)
    }
}

impl Display for TagKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

/// Represents a Wine GE or Wine GE LoL version.
///
/// Wine GE versions come in two flavours. One ist the normal Wine version with GE's patches and the other is a
/// specific version for League of Legends.
///
/// This enum supports `serde`'s serialization and deserialization traits.
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
#[serde(tag = "type")]
pub enum WineTagKind {
    WineGe,
    LolWineGe,
}

impl From<&str> for WineTagKind {
    fn from(string: &str) -> Self {
        match string {
            s if s.eq(WINE) => WineTagKind::WineGe,
            s if s.eq(LOL_WINE) => WineTagKind::LolWineGe,
            _ => panic!("Cannot map string to LutrisVersionKind"),
        }
    }
}

#[cfg(test)]
mod tag_tests {
    use test_case::test_case;

    use super::*;

    #[test_case("6.20-GE-1" => String::from("6.20.1"))]
    #[test_case("6.20-GE-0" => String::from("6.20.0"))]
    #[test_case("6.20-GE" => String::from("6.20.0"))]
    #[test_case("6.16-GE-3-LoL" => String::from("6.16.3-LoL"))]
    #[test_case("6.16-2-GE-LoL" => String::from("6.16.2-LoL"))]
    #[test_case("6.16-GE-LoL" => String::from("6.16.0-LoL"))]
    #[test_case("6.16-GE-0-LoL" => String::from("6.16.0-LoL"))]
    #[test_case("6.16-0-GE-LoL" => String::from("6.16.0-LoL"))]
    #[test_case("7.0rc3-GE-1" => String::from("7.0.1-rc3"))]
    #[test_case("7.0rc3-GE-0" => String::from("7.0.0-rc3"))]
    #[test_case("7.0rc3-GE" => String::from("7.0.0-rc3"))]
    #[test_case("7.0-GE" => String::from("7.0.0"))]
    #[test_case("7.0-GE-1" => String::from("7.0.1"))]
    #[test_case("GE-Proton7-8" => String::from("7.8.0"))]
    #[test_case("GE-Proton7-4" => String::from("7.4.0"))]
    #[test_case("5.11-GE-1-MF" => String::from("5.11.1-MF"))]
    #[test_case("proton-3.16-5" => String::from("3.16.5"))]
    #[test_case("5.0-rc5-GE-1" => String::from("5.0.1-rc5"))]
    fn get_semver_format(tag_str: &str) -> String {
        let tag = Tag::new(tag_str);
        tag.semver().to_string()
    }

    #[test]
    fn create_from_json_before_release_0_2_0() {
        let tag: Tag = serde_json::from_str(r###"{
            "value": "6.20-GE-1",
            "semver": {
                "major": 6, "minor": 20, "patch": 1, "identifier": null
            }
        }"###).unwrap();
        assert_eq!(tag.str(), "6.20-GE-1");
    }

    #[test]
    fn create_from_json() {
        let tag: Tag = serde_json::from_str(r###"{
            "str": "6.20-GE-1",
            "semver": {
                "major": 6, "minor": 20, "patch": 1, "identifier": null
            }
        }"###).unwrap();
        assert_eq!(tag.str(), "6.20-GE-1");
    }

    #[test_case(Tag::new("6.20-GE-1"), Tag::new("6.20-GE-1") => true)]
    #[test_case(Tag::new("6.20-GE-1"), Tag::new("6.21-GE-1") => false)]
    fn equality_tests(a: Tag, b: Tag) -> bool {
        a.eq(&b)
    }

    #[test_case(Tag::new("6.20-GE-1"), Tag::new("6.20-GE-1") => Ordering::Equal)]
    #[test_case(Tag::new("6.20-GE-1"), Tag::new("6.21-GE-1") => Ordering::Less)]
    #[test_case(Tag::new("6.20-GE-1"), Tag::new("6.19-GE-1") => Ordering::Greater)]
    #[test_case(Tag::new("GE-Proton7-8"), Tag::new("GE-Proton7-8") => Ordering::Equal)]
    #[test_case(Tag::new("GE-Proton7-8"), Tag::new("GE-Proton7-20") => Ordering::Less)]
    #[test_case(Tag::new("GE-Proton7-8"), Tag::new("GE-Proton7-7") => Ordering::Greater)]
    fn comparison_tests(a: Tag, b: Tag) -> Ordering {
        a.cmp(&b)
    }
}

#[cfg(test)]
mod tag_kind_tests {
    use test_case::test_case;

    use super::*;

    #[test]
    fn wine() {
        let kind = TagKind::wine();
        assert_eq!(
            kind,
            TagKind::Wine {
                kind: WineTagKind::WineGe
            }
        )
    }

    #[test]
    fn lol() {
        let kind = TagKind::lol();
        assert_eq!(
            kind,
            TagKind::Wine {
                kind: WineTagKind::LolWineGe
            }
        );
    }

    #[test]
    fn values() {
        let values = TagKind::values();
        assert_eq!(
            values,
            vec![
                TagKind::Proton,
                TagKind::Wine {
                    kind: WineTagKind::WineGe
                },
                TagKind::Wine {
                    kind: WineTagKind::LolWineGe
                },
            ]
        );
    }

    #[test_case(TagKind::Proton => "Proton GE"; "Correct app name should be returned for Proton")]
    #[test_case(TagKind::wine() => "Wine GE"; "Correct app name should be returned for Wine")]
    #[test_case(TagKind::lol() => "Wine GE (LoL)"; "Correct app name should be returned for Wine (LoL)")]
    fn get_compatibility_tool_name(kind: TagKind) -> String {
        kind.compatibility_tool_name()
    }

    #[test_case(TagKind::Proton => "PROTON"; "Correct type name should be returned for Proton")]
    #[test_case(TagKind::wine() => "WINE"; "Correct type name should be returned for Wine")]
    #[test_case(TagKind::lol() => "LOL_WINE"; "Correct type name should be returned for Wine (LoL)")]
    fn get_type_name(kind: TagKind) -> String {
        kind.str()
    }
}
