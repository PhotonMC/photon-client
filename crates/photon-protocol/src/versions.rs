//! What an older version's wire format does differently from the newest one, one file per
//! version ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)). The canonical
//! codecs in the other modules never name an older version; the few dispatch points that choose
//! between them are the only places a `Version` is compared.

pub mod v26_1;
