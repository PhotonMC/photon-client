# P01.d — VoxelShape join ve joinUnoptimized doğrulaması

`src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1903 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1903/1903** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2405/2405** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa`.

Birleşik referans kimliği: `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa`.

Yerel raporlar: `compare-xiw125c3` (math debug), `compare-v5h63z7q` (math release), `compare-chdvlz2_` (birleşik debug) ve `compare-oywxz0ky` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
