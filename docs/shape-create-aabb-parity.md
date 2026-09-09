# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 635 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **635/635** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1089/1089** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Birleşik referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Yerel raporlar: `compare-ihodhcd4` (math debug), `compare-1euc1ro8` (math release), `compare-_c9tjida` (birleşik debug) ve `compare-jbm02l6z` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-parity.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-parity.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
