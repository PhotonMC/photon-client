//! What an attribute declares (`Attributes`): its name and whether a rise
//! in it is good. Its description and its tooltip colours follow from
//! those, as the reference derives them.

use photon_core::version::{Version, VersionSpan};

/// `Attribute.Sentiment`: how a tooltip colours a rise and a fall.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

/// One attribute of the registry.
#[derive(Debug)]
pub struct Attribute {
    pub name: &'static str,
    pub sentiment: Sentiment,
    /// The versions that have the attribute.
    pub versions: VersionSpan,
}

impl Attribute {
    /// An attribute a rise in which is good (the default sentiment).
    pub const fn of(name: &'static str) -> Self {
        Self {
            name,
            sentiment: Sentiment::Positive,
            versions: VersionSpan::ALL,
        }
    }

    /// The first version that has the attribute.
    pub const fn since(mut self, version: Version) -> Self {
        self.versions = self.versions.since(version);
        self
    }

    /// Whether the attribute exists in `version`.
    pub const fn exists_in(&self, version: Version) -> bool {
        self.versions.contains(version)
    }

    pub const fn sentiment(mut self, sentiment: Sentiment) -> Self {
        self.sentiment = sentiment;
        self
    }

    /// The translation key of its name: `attribute.name.` and its path.
    pub fn description_id(&self) -> String {
        let path = self
            .name
            .split_once(':')
            .map_or(self.name, |(_, path)| path);
        format!("attribute.name.{path}")
    }

    /// The colours a rise and a fall of it show in.
    pub fn styles(&self) -> (&'static str, &'static str) {
        match self.sentiment {
            Sentiment::Positive => ("BLUE", "RED"),
            Sentiment::Negative => ("RED", "BLUE"),
            Sentiment::Neutral => ("GRAY", "GRAY"),
        }
    }
}
