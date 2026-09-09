# P01.d — AABB equals/hashCode doğrulaması

`src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 795 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **795/795** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1249/1249** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f`.

Yerel raporlar: `compare-1xrk6zz1` (math debug), `compare-qzmn2buo` (math release), `compare-h3m21imi` (birleşik debug) ve `compare-xnouubh6` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
