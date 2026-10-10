# T70j — Kalan blok animasyonları (2)

**Amaç:** [T70i](T70i-kalan-blok-animasyonlari.md)'nin kalanı: yaprakların yağmurdaki damlası (`Level.isRainingAt`), göz çiçeği (`EyeblossomBlock`), ateş böceği çalısı, solgun gül, asılı yosun, kuru ot ve ölü çalı sesleri, arı kovanı balı, creaking kalbi, end portalı ve geçidi, ıslak sünger dışında kalan su blokları, sarkıt ucu damlası (çevre öznitelikleri ister), kuru ghast, potent sulfur, fırçalanabilir blok, paratoner; yaratıcıda işaretçi (`BLOCK_MARKER`), biyomun ortam parçacıkları (`AMBIENT_PARTICLES`).
**Bağımlılık:** T70i. **Boy:** L.

## Oku
Java: ilgili blok sınıflarının `animateTick`'i. Rust: `crates/photon-world/src/particles/animate_redstone.rs` (örnek).

## Oracle
`movement/particle-engine` `animate_tick` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
(boş)
