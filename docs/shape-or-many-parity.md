# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1843 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1843/1843** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Yerel raporlar: `compare-5_avciij` (math debug), `compare-e6e6agr4` (math release), `compare-gvo3eind` (birleşik debug) ve `compare-wmabbl5t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
