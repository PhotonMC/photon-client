# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 695 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **695/695** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1149/1149** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Birleşik referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Yerel raporlar: `compare-z24w3ncm` (math debug), `compare-ue0l25nl` (math release), `compare-oiqblje5` (birleşik debug) ve `compare-pl0oao3t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
