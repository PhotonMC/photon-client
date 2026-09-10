# P01.d — AABB.expandTowards(Vec3) doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1378 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1378/1378** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81`.

Yerel raporlar: `compare-_ohp5wb8` (math debug), `compare-bezugwpw` (math release), `compare-i4o7vofo` (birleşik debug) ve `compare-9vhvfokc` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
