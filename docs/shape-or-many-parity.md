# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 995 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **995/995** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1497/1497** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f43f8cbd1b5228b300d98e13e54dbf07d01ae1691a4a16ba16b2e05745e9b4e9`.

Birleşik referans kimliği: `f43f8cbd1b5228b300d98e13e54dbf07d01ae1691a4a16ba16b2e05745e9b4e9`.

Yerel raporlar: `compare-wzdfn9wg` (math debug), `compare-p0nj33p_` (math release), `compare-sdhltqtk` (birleşik debug) ve `compare-af9v4abw` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
