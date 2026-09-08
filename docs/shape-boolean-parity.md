# P01.d — VoxelShape BooleanOp join doğrulaması

`src/shape.rs`, Java `BooleanOp.AND`, `ONLY_FIRST` ve `ONLY_SECOND` join sonuçlarını koordinat gridindeki hücreler üzerinden üretir. `examples/behavior_adapter.rs` aynı `Shapes.join` ve `Shapes.joinIsNotEmpty` gözlemlerini JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 349 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **349/349** math senaryosunda eşleşti.
- Dört yeni senaryoda örtüşen `AND`, ayrık `AND`, `ONLY_FIRST` çıkarma ve ayrık `ONLY_SECOND` sonucu ile `joinIsNotEmpty` değeri karşılaştırıldı.
- Birleşik altı aile corpus'u **803/803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3fbfb7e1e08bca037f422300c2080c84c0ab4608f3cd9c710e966f1c27d5085d`.

Birleşik referans kimliği: `9c6f5e56b8630fe1a153fe6933b0c6762ca3b2d4c58d807cd8faab447812bb3d`.

Yerel raporlar: `compare-kr2hii2y` (math debug), `compare-4s4cgekx` (math release), `compare-ry_cgnpg` (birleşik debug) ve `compare-nbf_jf87` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik BooleanOp senaryosuyla sınırlıdır. `BooleanOp` tablosundaki diğer işlemler, tam index merger seçenekleri, collision/occlusion ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
