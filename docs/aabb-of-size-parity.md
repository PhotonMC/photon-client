# P01.d — AABB ofSize doğrulaması

`src/aabb.rs`, Java `AABB.ofSize(Vec3, double, double, double)` overload'unda merkezden yarım boyut çıkarıp ekleyerek dört endpoint ifadesini üretir ve sonucu AABB constructor normalizasyonundan geçirir. `examples/behavior_adapter.rs` `of_size` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 430 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **430/430** math senaryosunda eşleşti.
- Altı yeni senaryoda birim ve asimetrik boyutlar, negatif ve sıfır boyutlar, signed zero, sonsuzluk ve NaN karşılaştırıldı.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c067c0c2f0322ab2784a92ca8072c3585d3c8c0367c9e6bce26190d0f22adf75`.

Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Yerel raporlar: `compare-8qoc7rvw` (math debug), `compare-_2e59bfu` (math release), `compare-94x3ynna` (birleşik debug) ve `compare-2cii6gte` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.ofSize(Vec3, double, double, double)` overload'u ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; Çoklu AABB clip sonraki dilimdir.
