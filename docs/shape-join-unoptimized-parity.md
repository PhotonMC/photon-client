# P01.d — VoxelShape join ve joinUnoptimized doğrulaması

`src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 685 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **685/685** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1139/1139** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8`.

Birleşik referans kimliği: `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8`.

Yerel raporlar: `compare-2oxv9wmk` (math debug), `compare-54arzdz4` (math release), `compare-3tby77d3` (birleşik debug) ve `compare-nscg_s6g` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
