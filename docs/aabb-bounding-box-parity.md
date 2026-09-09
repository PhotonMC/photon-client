# P01.d — AABB BoundingBox köprüsü doğrulaması

`src/aabb.rs`, Java `BoundingBox` int sınırlarını normalize edip `AABB.of(BoundingBox)` üreticisinin üst uçlardaki Java int `+1` işlemini uygular. `examples/behavior_adapter.rs` bu yolu JSONL math senaryolarına bağlar.

## Referans ve davranış

Java 26.2 / protokol 776 / Java 25 referansı, `AABB.java:39-41` ve `BoundingBox.java:48-64,288-318` kaynak yollarıyla kullanıldı. Ayrıntılı davranış ve altı senaryonun girdileri [sözleşme belgesindedir](aabb-bounding-box-contract.md). Ters sınır senaryoları için referans classpath'ına sabitlenmiş `brigadier-1.3.10.jar` eklendi; bu yalnızca Java'nın constructor uyarı yolunun çalışabilmesi içindir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 474 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **474/474** math senaryosunda eşleşti.
- Altı yeni senaryoda normal sınırlar, tümü ve seçili eksenleri ters sınırlar, `Integer.MAX_VALUE + 1` taşması, iki tamsayı ucu ve ardışık üretici çağrıları karşılaştırıldı.
- Altı ailenin birleşik corpus'u **1049/1049** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5cb113d71b4ba206c07ea73382625520d811f4ecd4bef0769e15bdd55427a0b4`.

Birleşik referans kimliği: `da26bf4827a53d92d2e5f34a0eef95d42ce2a999192fdf5c8181ec7a0eb1ed89`.

Yerel raporlar: `compare-_1vm5zsh` (math debug), `compare-emu4ivo7` (math release), `compare-b4pkrz59` (birleşik debug) ve `compare-dcvn_t28` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `BoundingBox` altı int endpoint constructor'ı, `AABB.of(BoundingBox)` ve mevcut AABB adım adaptörüyle sınırlıdır. VoxelShape clip tamamlandı; occlusion ve istemci hedefleme davranışları sonraki dilimlerdir.
