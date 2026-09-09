# P01.d — VoxelShape join ve joinUnoptimized doğrulaması

`src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1046 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1046/1046** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Birleşik referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Yerel raporlar: `compare-m8v98wom` (math debug), `compare-4i19c_i3` (math release), `compare-p7m9abjy` (birleşik debug) ve `compare-vvlbkmvf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
