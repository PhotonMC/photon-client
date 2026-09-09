# P01.d — Shapes.create(AABB) doğrulaması

`src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 610 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **610/610** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1064/1064** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `6237f08207a433a01af674f3c2eba4861897d31ba0a6743d00eac9be068d09a9`.

Birleşik referans kimliği: `298fc79fff700683b37658552439ad616697eca9abe5ae8a7cd43d2e28ce910e`.

Yerel raporlar: `compare-pjrdg23d` (math debug), `compare-pl_ri3o5` (math release), `compare-45pvo1ui` (birleşik debug) ve `compare-bodmv5fk` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. Scalar `Shapes.create(double...)` için ayrı public Rust adı, `Shapes.INFINITY` sabiti ve shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
