# P01.d — Shapes.rotateAll merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, rotationCenter)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön map'iyle uygular; özel merkezi rotate çağrılarına iletir ve Java `EnumMap` sırasını korur. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 592 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **592/592** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön map'ini, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- NORTH yönünün başlangıç `IDENTITY` dönüşüyle özel merkezi kullanmadan özgün shape'i koruması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1046/1046** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a96f6ac711181f55389712d207cddb3ac84e94a0b65b0aa592874d653bb8d9fd`.

Birleşik referans kimliği: `b23e0bd7aa5f9df5baa9862e79e09f08f309c7b06b36028da91d501f3ddae578`.

Yerel raporlar: `compare-fu6yzw05` (math debug), `compare-jp_d2q8j` (math release), `compare-ixe0azli` (birleşik debug) ve `compare-k8ldje4k` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` başlangıç dönüşlü iki argümanlı overload, üç sentetik box ve altı yön map'iyle sınırlıdır. `initial` rotation alan overload, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
