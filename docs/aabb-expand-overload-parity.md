# P01.d — AABB.expandTowards(Vec3) doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1407/1407** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `a2c69ab98e59ffb82410fd45282bc6e8c110fd13877ba87471c7bfcc29f23318`.

Yerel raporlar: `compare-psz_s0y8` (math debug), `compare-xhpjkx1_` (math release), `compare-smgo22vh` (birleşik debug) ve `compare-h2kr9uj0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
