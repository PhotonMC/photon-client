# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 650 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **650/650** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1104/1104** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Birleşik referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Yerel raporlar: `compare-krjmldv0` (math debug), `compare-ufbmlufj` (math release), `compare-o61mnrf6` (birleşik debug) ve `compare-6fi73fsy` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-parity.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-parity.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
