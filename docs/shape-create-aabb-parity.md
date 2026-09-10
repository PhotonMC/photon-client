# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1987 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug) ve `compare-jqj2kta_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-parity.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-parity.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
