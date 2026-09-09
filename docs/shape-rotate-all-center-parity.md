# P01.d — Shapes.rotateAll merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, rotationCenter)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön map'iyle uygular; özel merkezi rotate çağrılarına iletir ve Java `EnumMap` sırasını korur. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 571 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **571/571** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön map'ini, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- NORTH yönünün başlangıç `IDENTITY` dönüşüyle özel merkezi kullanmadan özgün shape'i koruması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1025/1025** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `bf8eac3fe88ccc7eac0052e8b44cb5dab3dfb456982cd6a700ad1f08fadc02a2`.

Birleşik referans kimliği: `3133f0b82eddafd4383cd19cad3d4e53db2b9051614cfc78968ee4d6d98810b8`.

Yerel raporlar: `compare-wbo29fbq` (math debug), `compare-7m5csdkl` (math release), `compare-tyoie69t` (birleşik debug) ve `compare-_lv2ppe_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` başlangıç dönüşlü iki argümanlı overload, üç sentetik box ve altı yön map'iyle sınırlıdır. `initial` rotation alan overload, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
