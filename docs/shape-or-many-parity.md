# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2023 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2023/2023** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Yerel raporlar: `compare-4sx1lm2m` (math debug), `compare-m3ib6sjn` (math release), `compare-czvhou3w` (birleşik debug) ve `compare-oz19j96e` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
