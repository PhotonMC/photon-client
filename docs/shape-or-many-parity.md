# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 627 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **627/627** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1081/1081** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Birleşik referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Yerel raporlar: `compare-tg0rl658` (math debug), `compare-713odvz2` (math release), `compare-pnvz2myv` (birleşik debug) ve `compare-_bmqdjjb` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
