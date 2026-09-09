# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 695 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **695/695** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1149/1149** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Birleşik referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Yerel raporlar: `compare-z24w3ncm` (math debug), `compare-ue0l25nl` (math release), `compare-oiqblje5` (birleşik debug) ve `compare-pl0oao3t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-parity.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-parity.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
