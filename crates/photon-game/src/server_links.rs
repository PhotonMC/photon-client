//! `ServerLinks`: the addresses a server offers the player (`ClientboundServerLinksPacket`), and
//! the check the client makes of each before it trusts it.

use crate::text::Component;

/// `ServerLinks.KnownLinkType`, by name; an id out of range is the first (`ByIdMap` zero).
const KNOWN_TYPES: [&str; 10] = [
    "report_bug",
    "community_guidelines",
    "support",
    "status",
    "feedback",
    "community",
    "website",
    "forums",
    "news",
    "announcements",
];

/// What names a link: one of the known types, or a component the server wrote.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Known(usize),
    Custom(Component),
}

impl LinkType {
    /// A known type from its wire id (`ByIdMap.OutOfBoundsStrategy.ZERO`).
    pub fn known(id: i32) -> Self {
        Self::Known(
            usize::try_from(id)
                .ok()
                .filter(|id| *id < KNOWN_TYPES.len())
                .unwrap_or(0),
        )
    }

    /// `Entry.displayName`: the type's own translation, or the component.
    pub fn display_name(&self) -> Component {
        match self {
            Self::Known(id) => {
                Component::translatable(format!("known_server_link.{}", KNOWN_TYPES[*id]))
            }
            Self::Custom(component) => component.clone(),
        }
    }
}

/// `ServerLinks.UntrustedEntry`: as the packet carried it.
#[derive(Debug, Clone, PartialEq)]
pub struct UntrustedEntry {
    pub link_type: LinkType,
    pub link: String,
}

/// `ServerLinks.Entry`: a link that passed `Util.parseAndValidateUntrustedUri`.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub link_type: LinkType,
    pub link: String,
}

/// `ServerLinks`: the entries the client trusts, in the order the server listed them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerLinks {
    entries: Vec<Entry>,
}

impl ServerLinks {
    /// `ClientCommonPacketListenerImpl.handleServerLinks`: an entry whose address is not valid is
    /// left out.
    pub fn trusted(untrusted: Vec<UntrustedEntry>) -> Self {
        Self {
            entries: untrusted
                .into_iter()
                .filter(|entry| is_valid_untrusted_uri(&entry.link))
                .map(|entry| Entry {
                    link_type: entry.link_type,
                    link: entry.link,
                })
                .collect(),
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `Util.parseAndValidateUntrustedUri`: a `java.net.URI` that has a scheme of `http` or `https`.
/// The syntax check covers what the class refuses outright (characters no part allows, a stray
/// percent sign, a scheme that is not one); the finer rules of its authority are not read.
pub fn is_valid_untrusted_uri(uri: &str) -> bool {
    let Some((scheme, rest)) = uri.split_once(':') else {
        return false;
    };
    is_web_scheme(scheme) && has_only_uri_characters(rest)
}

/// A scheme as the URI syntax spells one, and one of the two a link may use.
fn is_web_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let spelled = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    spelled && matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https")
}

/// What follows the scheme holds no character the class refuses: the ones no part allows, a
/// percent sign without two hex digits behind it, and brackets outside the authority.
fn has_only_uri_characters(rest: &str) -> bool {
    let authority_end = rest.strip_prefix("//").map_or(0, |after| {
        2 + after.find(['/', '?', '#']).unwrap_or(after.len())
    });
    let bytes = rest.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let allowed = match bytes[index] {
            b'%' => {
                let hex = |at: usize| bytes.get(at).is_some_and(u8::is_ascii_hexdigit);
                if hex(index + 1) && hex(index + 2) {
                    index += 2;
                    true
                } else {
                    false
                }
            }
            b'[' | b']' => index < authority_end,
            b' ' | b'"' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}' => false,
            0..=0x1f | 0x7f => false,
            _ => true,
        };
        if !allowed {
            return false;
        }
        index += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_and_https_addresses_are_trusted() {
        for good in [
            "https://example.com/a?b=c#d",
            "HTTP://example.com",
            "http://[::1]:25565/",
        ] {
            assert!(is_valid_untrusted_uri(good), "{good}");
        }
        for bad in [
            "example.com",
            "ftp://example.com",
            "javascript:alert(1)",
            "https://exa mple.com",
            "https://example.com/%zz",
            "https://example.com/a|b",
            "://x",
        ] {
            assert!(!is_valid_untrusted_uri(bad), "{bad}");
        }
    }

    #[test]
    fn a_known_type_out_of_range_is_the_first_and_names_its_translation() {
        assert_eq!(LinkType::known(99), LinkType::Known(0));
        assert_eq!(LinkType::known(-1), LinkType::Known(0));
        let name = LinkType::known(9).display_name();
        assert_eq!(
            name,
            Component::translatable("known_server_link.announcements")
        );
    }

    #[test]
    fn invalid_entries_are_left_out_and_the_order_is_kept() {
        let entry = |link: &str| UntrustedEntry {
            link_type: LinkType::known(2),
            link: link.to_owned(),
        };
        let links = ServerLinks::trusted(vec![
            entry("https://a.example"),
            entry("nope"),
            entry("http://b.example"),
        ]);
        let kept: Vec<&str> = links.entries().iter().map(|e| e.link.as_str()).collect();
        assert_eq!(kept, ["https://a.example", "http://b.example"]);
    }
}
