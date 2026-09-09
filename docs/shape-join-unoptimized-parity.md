# P01.d — VoxelShape join ve joinUnoptimized doğrulaması

`src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 604 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **604/604** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1058/1058** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `ee01fb100a4d138a2dc2967dc19d79f7e79a055fec252d406d23599850925ae3`.

Birleşik referans kimliği: `b251a60111607ad28c03f7396010655e88be4532d7dc3f048d4f3db054fce03e`.

Yerel raporlar: `compare-v217glqz` (math debug), `compare-snqwxxvj` (math release), `compare-ngv2kroy` (birleşik debug) ve `compare-r5i49tld` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
