# P01.d — AABB.expandTowards(Vec3) doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 973 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **973/973** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1475/1475** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `6be1cb26871cc94c222742a0c5ccb50c6ef5df50a6b4f33206aeefdd805d2a60`.

Yerel raporlar: `compare-yzj8g47` (math debug), `compare-0gyul_vz` (math release), `compare-1qxbdgol` (birleşik debug) ve `compare-9okhsl5l` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
