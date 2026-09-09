# P01.d — AABB iterable clip ve BlockHitResult doğrulaması

`src/aabb.rs`, Java iterable `AABB.clip` overloadunun aday kutuları `BlockPos` ile taşımasını, ortak en yakın segment ölçeğini, yüz yönünü ve `BlockHitResult` alanlarını uygular. `examples/behavior_adapter.rs` bu yolu JSONL math senaryolarına bağlar.

## Referans ve davranış

Java 26.2 / protokol 776 / Java 25 referansı, `AABB.java:314-388` ve `BlockHitResult.java:6-65` kaynak yollarıyla kullanıldı. Ayrıntılı davranış ve dokuz senaryonun girdileri [sözleşme belgesindedir](aabb-clip-many-contract.md).

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 483 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **483/483** math senaryosunda eşleşti.
- Dokuz yeni senaryoda altı yüz yönü, köşe yön önceliği, aday liste taraması, BlockPos taşıması, boş/miss sonuçları ve başlangıç/eşik davranışı karşılaştırıldı.
- Altı ailenin birleşik corpus'u **1010/1010** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `574f55bea348590f76cc3685cd87aadd677ba8f4649d2acfa055ee6138565743`.

Birleşik referans kimliği: `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c`.

Yerel raporlar: `compare-e5jzlrsg` (math debug), `compare-odylvazt` (math release), `compare-sq3wafy1` (birleşik debug) ve `compare-1i1dj3ib` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iterable AABB clip overloadu ve sabit `BlockHitResult` constructor alanlarıyla sınırlıdır. VoxelShape clip tamamlandı; occlusion, entity hit sonuçları ve istemci hedefleme sırası sonraki dilimlerdir.
