const KEYWORDS_MSVC: [&str; 1] = ["msvc"];
const KEYWORDS_GNU: [&str; 1] = ["gnu"];
const KEYWORDS_MUSL: [&str; 1] = ["musl"];

/**
    Enum representing a system toolchain, such as MSVC or GNU.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Toolchain {
    Msvc,
    Gnu,
    Musl,
}

impl Toolchain {
    /**
        Get the toolchain of the current host system.
    */
    pub fn current() -> Option<Self> {
        None // TODO: Implement detection of the host toolchain
    }

    /**
        Detect a toolchain by identifying keywords in a search string.
    */
    pub fn detect(search_string: impl AsRef<str>) -> Option<Self> {
        let lowercased = search_string.as_ref().to_lowercase();
        for keyword in &KEYWORDS_MSVC {
            if lowercased.contains(keyword) {
                return Some(Self::Msvc);
            }
        }
        for keyword in &KEYWORDS_GNU {
            if lowercased.contains(keyword) {
                return Some(Self::Gnu);
            }
        }
        for keyword in &KEYWORDS_MUSL {
            if lowercased.contains(keyword) {
                return Some(Self::Musl);
            }
        }
        None
    }
}