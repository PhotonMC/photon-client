# P01.d — AABB hasNaN doğrulaması

`src/aabb.rs`, Java `AABB.hasNaN()` sorgusunun altı saklanan endpoint'i `is_nan` ile kontrol etmesini uygular. `examples/behavior_adapter.rs` `has_nan` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 406 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **406/406** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu kutu, üç NaN yerleşimi ve pozitif/negatif sonsuzluk karşılaştırıldı.
- Birleşik altı aile corpus'u **997/997** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `1b628a5ed1aa6d5d7f23933491eed86b44ffdbc00f80560b0ddbe51afb57e990`.

Birleşik referans kimliği: `d12db998718af31adab6351bbe129bcfd8d13fb33be8aa4270b7bc0546294c2d`.

Yerel raporlar: `compare-cvrcdo06` (math debug), `compare-t0m86qa6` (math release), `compare-m_cdl8w8` (birleşik debug) ve `compare-xp7qipgf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.hasNaN()` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; AABB move overload'ları [ayrı kanıtta tamamlandı](aabb-move-overloads-parity.md); [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
