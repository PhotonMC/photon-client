//! The saved servers (`ServerList`, `servers.dat` in the game directory): the ones the list shows,
//! in order, and the hidden ones a direct connection remembers.

use std::path::Path;

use photon_core::nbt::{Field, FieldValue};

/// `ServerList.MAX_HIDDEN_SERVERS`.
const MAX_HIDDEN: usize = 16;

/// `ServerData.ServerPackStatus`: whether the server's resource pack is taken without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PackStatus {
    Enabled,
    Disabled,
    #[default]
    Prompt,
}

/// What `servers.dat` keeps of a server (`ServerData.write`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerData {
    pub name: String,
    pub ip: String,
    /// The favicon as the file keeps it, base64.
    pub icon: Option<String>,
    pub pack_status: PackStatus,
    pub accepted_code_of_conduct: i32,
}

impl ServerData {
    pub fn new(name: &str, ip: &str) -> Self {
        Self {
            name: name.to_owned(),
            ip: ip.to_owned(),
            ..Self::default()
        }
    }

    /// `ServerData.getIconBytes`.
    pub fn icon_bytes(&self) -> Option<Vec<u8>> {
        self.icon.as_deref().and_then(photon_core::base64::decode)
    }

    /// `ServerData.setIconBytes`.
    pub fn set_icon_bytes(&mut self, bytes: Option<&[u8]>) {
        self.icon = bytes.map(photon_core::base64::encode);
    }

    /// What `handleStatusResponse` does with an answer's favicon: a different one replaces the
    /// saved icon, kept only when `validate_icon` takes it. Whether the list needs saving.
    pub fn apply_ping_icon(&mut self, favicon: &[u8]) -> bool {
        if self.icon_bytes().as_deref() == Some(favicon) {
            return false;
        }
        self.set_icon_bytes(validate_icon(favicon));
        true
    }

    /// `ServerData.read`.
    fn read(fields: &[Field]) -> Self {
        let string = |key: &str| match field(fields, key) {
            Some(FieldValue::Str(value)) => Some(value.clone()),
            _ => None,
        };
        Self {
            name: string("name").unwrap_or_default(),
            ip: string("ip").unwrap_or_default(),
            icon: string("icon").filter(|text| photon_core::base64::decode(text).is_some()),
            pack_status: match field(fields, "acceptTextures") {
                Some(FieldValue::Byte(0)) => PackStatus::Disabled,
                Some(FieldValue::Byte(_)) => PackStatus::Enabled,
                _ => PackStatus::Prompt,
            },
            accepted_code_of_conduct: match field(fields, "acceptedCodeOfConduct") {
                Some(FieldValue::Int(value)) => *value,
                _ => 0,
            },
        }
    }

    /// `ServerData.write` and the list's `hidden` flag.
    fn write(&self, hidden: bool) -> FieldValue {
        let entry = |key: &str, value| Field {
            key: key.to_owned(),
            value,
        };
        let mut fields = vec![
            entry("name", FieldValue::Str(self.name.clone())),
            entry("ip", FieldValue::Str(self.ip.clone())),
        ];
        if let Some(icon) = &self.icon {
            fields.push(entry("icon", FieldValue::Str(icon.clone())));
        }
        match self.pack_status {
            PackStatus::Enabled => fields.push(entry("acceptTextures", FieldValue::Byte(1))),
            PackStatus::Disabled => fields.push(entry("acceptTextures", FieldValue::Byte(0))),
            PackStatus::Prompt => {}
        }
        if self.accepted_code_of_conduct != 0 {
            fields.push(entry(
                "acceptedCodeOfConduct",
                FieldValue::Int(self.accepted_code_of_conduct),
            ));
        }
        fields.push(entry("hidden", FieldValue::Byte(i8::from(hidden))));
        FieldValue::Compound(fields)
    }
}

/// `ServerData.validateIcon`: a PNG no bigger than 1024 by 1024, judged by its header alone.
pub fn validate_icon(bytes: &[u8]) -> Option<&[u8]> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
    let header = bytes.get(..24)?;
    let size = |at: usize| {
        u32::from_be_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]])
    };
    // Java reads the width and height as signed ints; a negative one is under the limit.
    let fits = |value: u32| (value as i32) <= 1024;
    (header[..8] == SIGNATURE
        && size(8) == 13
        && header[12..16] == *b"IHDR"
        && fits(size(16))
        && fits(size(20)))
    .then_some(bytes)
}

fn field<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| &field.value)
}

/// `ServerList`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerList {
    pub servers: Vec<ServerData>,
    pub hidden: Vec<ServerData>,
}

impl ServerList {
    /// `ServerList.load`: none when the file is missing or unreadable.
    pub fn load(game_directory: &Path) -> Self {
        std::fs::read(game_directory.join("servers.dat"))
            .ok()
            .and_then(|bytes| Self::from_file_bytes(&bytes))
            .unwrap_or_default()
    }

    /// The list from the bytes of a `servers.dat` (`NbtIo.read`: an uncompressed named root).
    pub fn from_file_bytes(bytes: &[u8]) -> Option<Self> {
        let (&id, rest) = bytes.split_first()?;
        let name_length = usize::from(u16::from_be_bytes([*rest.first()?, *rest.get(1)?]));
        let payload = rest.get(2 + name_length..)?;
        let unnamed = [&[id][..], payload].concat();
        let root = photon_core::nbt::read_nbt(&mut unnamed.as_slice()).ok()??;
        let mut list = Self::default();
        if let Some(FieldValue::List(entries)) = field(&root, "servers") {
            for entry in entries {
                let FieldValue::Compound(fields) = entry else {
                    continue;
                };
                let hidden =
                    matches!(field(fields, "hidden"), Some(FieldValue::Byte(value)) if *value != 0);
                let server = ServerData::read(fields);
                if hidden {
                    list.hidden.push(server);
                } else {
                    list.servers.push(server);
                }
            }
        }
        Some(list)
    }

    /// The bytes `ServerList.save` writes: the shown servers, then the hidden ones.
    pub fn to_file_bytes(&self) -> Vec<u8> {
        let entries = self
            .servers
            .iter()
            .map(|server| server.write(false))
            .chain(self.hidden.iter().map(|server| server.write(true)))
            .collect();
        let root = [Field {
            key: "servers".to_owned(),
            value: FieldValue::List(entries),
        }];
        let mut unnamed = Vec::new();
        photon_core::nbt::write_nbt(Some(&root), &mut unnamed);
        // A file's root has a name, empty.
        let mut bytes = vec![unnamed[0], 0, 0];
        bytes.extend_from_slice(&unnamed[1..]);
        bytes
    }

    /// `ServerList.save`: the old file kept as `servers.dat_old`.
    pub fn save(&self, game_directory: &Path) -> std::io::Result<()> {
        let current = game_directory.join("servers.dat");
        let fresh = game_directory.join("servers.dat_new");
        std::fs::write(&fresh, self.to_file_bytes())?;
        if current.exists() {
            std::fs::rename(&current, game_directory.join("servers.dat_old"))?;
        }
        std::fs::rename(fresh, current)
    }

    /// `ServerList.get(ip)`: a shown server first, then a hidden one.
    pub fn find(&self, ip: &str) -> Option<&ServerData> {
        self.servers
            .iter()
            .chain(&self.hidden)
            .find(|server| server.ip == ip)
    }

    /// `ServerList.add`: a hidden one goes first and the oldest past sixteen are forgotten.
    pub fn add(&mut self, server: ServerData, hidden: bool) {
        if hidden {
            self.hidden.insert(0, server);
            self.hidden.truncate(MAX_HIDDEN);
        } else {
            self.servers.push(server);
        }
    }

    /// `ServerList.unhide`: a hidden server of this address moves to the end of the shown ones.
    pub fn unhide(&mut self, ip: &str) -> Option<&mut ServerData> {
        let index = self.hidden.iter().position(|server| server.ip == ip)?;
        let server = self.hidden.remove(index);
        self.servers.push(server);
        self.servers.last_mut()
    }

    /// `ServerList.swap`.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.servers.swap(a, b);
    }
}

/// `ServerAddress.parseString` with its default port: `host:port` for the address resolver
/// (the reference also asks the name's SRV record when no port is given; this does not).
pub fn address_with_port(ip: &str) -> String {
    let ip = ip.trim();
    let has_port = match ip.rfind(']') {
        Some(close) => ip[close..].contains(':'),
        None => ip.matches(':').count() == 1,
    };
    if has_port {
        ip.to_owned()
    } else if ip.contains(':') && !ip.starts_with('[') {
        format!("[{ip}]:25565")
    } else {
        format!("{ip}:25565")
    }
}

/// `ServerAddress.isValidAddress`: Guava's `HostAndPort.fromString` takes it (a bracketed or
/// bare IPv6 host, else `host[:port]` with a port of digits up to 65535) and the host is not
/// empty, nor a name `IDN.toASCII` refuses (a label longer than 63).
pub fn is_valid_address(input: &str) -> bool {
    let host = if let Some(rest) = input.strip_prefix('[') {
        let Some(close) = rest.find(']') else {
            return false;
        };
        let after = &rest[close + 1..];
        if !after.is_empty() && !after.strip_prefix(':').is_some_and(valid_port) {
            return false;
        }
        &rest[..close]
    } else {
        match input.split_once(':') {
            Some((host, port)) if !port.contains(':') => {
                if !valid_port(port) {
                    return false;
                }
                host
            }
            _ => input,
        }
    };
    !host.is_empty() && host.split('.').all(|label| label.chars().count() <= 63)
}

/// Guava's port: ASCII digits (no sign) of a number up to 65535.
fn valid_port(port: &str) -> bool {
    !port.is_empty()
        && port.bytes().all(|byte| byte.is_ascii_digit())
        && port.parse::<u32>().is_ok_and(|port| port <= 65_535)
}

#[cfg(test)]
mod tests {
    #[test]
    fn addresses_are_checked_as_the_reference_does() {
        use super::is_valid_address as valid;
        assert!(valid("example.com") && valid("127.0.0.1:25600") && valid("[::1]:25565"));
        assert!(valid("::1") && valid("[::1]"));
        assert!(!valid("") && !valid(":25565") && !valid("host:") && !valid("host:+1"));
        assert!(!valid("host:65536") && !valid("[::1") && !valid("[::1]x"));
        assert!(!valid(&"a".repeat(64)));
    }

    #[test]
    fn an_address_gets_the_default_port() {
        assert_eq!(super::address_with_port("example.com"), "example.com:25565");
        assert_eq!(
            super::address_with_port("127.0.0.1:25600"),
            "127.0.0.1:25600"
        );
        assert_eq!(super::address_with_port("[::1]"), "[::1]:25565");
        assert_eq!(super::address_with_port("::1"), "[::1]:25565");
    }

    use super::*;

    /// The header of a PNG of `width` by `height`.
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n', 0, 0, 0, 13,
        ];
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn icons_are_kept_up_to_1024_by_1024() {
        assert!(validate_icon(&png_header(64, 64)).is_some());
        assert!(validate_icon(&png_header(1024, 1024)).is_some());
        assert!(validate_icon(&png_header(1025, 1)).is_none());
        assert!(validate_icon(&png_header(1, 1025)).is_none());
        assert!(validate_icon(b"not a png at all, just text").is_none());
        assert!(validate_icon(&png_header(64, 64)[..20]).is_none());
    }

    #[test]
    fn a_pinged_icon_replaces_the_saved_one_once() {
        let mut server = ServerData::new("a", "b");
        let icon = png_header(64, 64);
        assert!(server.apply_ping_icon(&icon));
        assert_eq!(server.icon_bytes(), Some(icon.clone()));
        assert!(!server.apply_ping_icon(&icon));
        // An icon past the limit is a change that leaves no icon.
        assert!(server.apply_ping_icon(&png_header(2000, 2000)));
        assert_eq!(server.icon, None);
    }

    #[test]
    fn a_saved_list_reads_back() {
        let mut list = ServerList::default();
        list.add(ServerData::new("A", "a.example"), false);
        list.add(
            ServerData {
                pack_status: PackStatus::Disabled,
                accepted_code_of_conduct: 3,
                ..ServerData::new("B", "b.example:25600")
            },
            false,
        );
        list.add(ServerData::new("Direct", "c.example"), true);
        assert_eq!(
            ServerList::from_file_bytes(&list.to_file_bytes()),
            Some(list)
        );
    }

    #[test]
    fn hidden_servers_keep_sixteen_newest_first() {
        let mut list = ServerList::default();
        for index in 0..20 {
            list.add(ServerData::new("", &index.to_string()), true);
        }
        assert_eq!(list.hidden.len(), 16);
        assert_eq!(list.hidden[0].ip, "19");
    }
}
