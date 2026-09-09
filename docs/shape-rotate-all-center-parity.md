# P01.d — Shapes.rotateAll merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, rotationCenter)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön map'iyle uygular; özel merkezi rotate çağrılarına iletir ve Java `EnumMap` sırasını korur. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön map'ini, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- NORTH yönünün başlangıç `IDENTITY` dönüşüyle özel merkezi kullanmadan özgün shape'i koruması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Birleşik referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug) ve `compare-4_se5ekn` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` başlangıç dönüşlü iki argümanlı overload, üç sentetik box ve altı yön map'iyle sınırlıdır. `initial` rotation alan overload, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
