# P01.d — Shapes.or varargs doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.or(first, tail...)` overload'ını `Shapes::or_many` ile soldan sağa binary OR birleşimi olarak uygular. `tests/behavior/java/MathOracle.java` Java varargs çağrısını, `examples/behavior_adapter.rs` ise aynı ilk+tail girdisini Rust API'sine bağlar. Davranış sınırları [varargs sözleşmesinde](shape-or-many-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 592 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **592/592** math senaryosunda eşleşti.
- Üç box'ın touching, disjoint ve empty-first varargs sıraları ile Java `toAabbs` snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1046/1046** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a96f6ac711181f55389712d207cddb3ac84e94a0b65b0aa592874d653bb8d9fd`.

Birleşik referans kimliği: `b23e0bd7aa5f9df5baa9862e79e09f08f309c7b06b36028da91d501f3ddae578`.

Yerel raporlar: `compare-fu6yzw05` (math debug), `compare-jp_d2q8j` (math release), `compare-ixe0azli` (birleşik debug) ve `compare-k8ldje4k` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box'tan oluşan varargs listeleriyle sınırlıdır. Daha uzun listeler, diğer BooleanOp yolları ve world collider toplama ayrı kapsamdır.
