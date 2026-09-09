# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 777 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **777/777** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1231/1231** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe`.

Birleşik referans kimliği: `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe`.

Yerel raporlar: `compare-da6k05zu` (math debug), `compare-jem8yqw3` (math release), `compare-r10lr0oe` (birleşik debug) ve `compare-rb0k9jx5` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-parity.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-parity.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
