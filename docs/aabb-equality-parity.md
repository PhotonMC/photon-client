# P01.d — AABB equals/hashCode doğrulaması

`src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 935 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **935/935** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1437/1437** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a`.

Yerel raporlar: `compare-96_s862r` (math debug), `compare-cbwrzmb5` (math release), `compare-fn36ocz_` (birleşik debug) ve `compare-a_hggp11` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
