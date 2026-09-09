# P01.d — AABB equals/hashCode doğrulaması

`src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 765 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **765/765** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1219/1219** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a`.

Yerel raporlar: `compare-ya8jvd1o` (math debug), `compare-rh0c0wxr` (math release), `compare-u_e3yyct` (birleşik debug) ve `compare-326ehkgq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
