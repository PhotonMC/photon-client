# P01.d — AABB equals/hashCode doğrulaması

`src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 747 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **747/747** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1201/1201** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `faa59d430a7f5ffebd3b634a6e1cb6970d92ff920794ed661c7eaf6175f68fb7`.

Yerel raporlar: `compare-id7epnq0` (math debug), `compare-otdg_h7s` (math release), `compare-loq4vhwu` (birleşik debug) ve `compare-d1k2vrim` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
