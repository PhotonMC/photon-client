# P01.d — VoxelShape join ve joinUnoptimized doğrulaması

`src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 635 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **635/635** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1089/1089** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Birleşik referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Yerel raporlar: `compare-ihodhcd4` (math debug), `compare-1euc1ro8` (math release), `compare-_c9tjida` (birleşik debug) ve `compare-jbm02l6z` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
