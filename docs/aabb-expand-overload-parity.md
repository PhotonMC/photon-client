# P01.d — AABB.expandTowards(Vec3) doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 747 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **747/747** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1201/1201** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `faa59d430a7f5ffebd3b634a6e1cb6970d92ff920794ed661c7eaf6175f68fb7`.

Yerel raporlar: `compare-id7epnq0` (math debug), `compare-otdg_h7s` (math release), `compare-loq4vhwu` (birleşik debug) ve `compare-d1k2vrim` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
