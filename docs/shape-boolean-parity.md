# P01.d — VoxelShape BooleanOp join doğrulaması

`src/shape.rs`, Java'nın `Shapes.join` tarafından kabul ettiği sekiz BooleanOp sonucunu koordinat gridindeki hücreler üzerinden üretir. `examples/behavior_adapter.rs` aynı `Shapes.join` ve `Shapes.joinIsNotEmpty` gözlemlerini JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 361 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **361/361** math senaryosunda eşleşti.
- Sekiz shape BooleanOp senaryosunda `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `FALSE`, `NOT_SAME`, `SECOND` ve `FIRST` sonuçları ile `joinIsNotEmpty` değeri karşılaştırıldı; OR temel slice içinde ayrıca doğrulandı.
- Birleşik altı aile corpus'u **1313/1313** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a76b7eaed8646c9469410bc3c88b48be54654ae35d49dcea4d88d7e3ba8904b7`.

Birleşik referans kimliği: `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2`.

Yerel raporlar: `compare-c_grkisa` (math debug), `compare-ajjamzol` (math release), `compare-6n7i4k79` (birleşik debug) ve `compare-se85bnok` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik ve box tabanlı BooleanOp senaryosuyla sınırlıdır. `Shapes.join` tarafından reddedilen BooleanOp sabitleri, tam index merger seçenekleri, collision/occlusion ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
