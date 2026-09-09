# P01.d — AABB.expandTowards(Vec3) doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1007 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1007/1007** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug) ve `compare-a2h2wcm2` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
