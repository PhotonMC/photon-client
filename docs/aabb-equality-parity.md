# P01.d — AABB equals/hashCode doğrulaması

`src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 777 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **777/777** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1231/1231** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe`.

Yerel raporlar: `compare-da6k05zu` (math debug), `compare-jem8yqw3` (math release), `compare-r10lr0oe` (birleşik debug) ve `compare-rb0k9jx5` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
