# P01.d — Shapes.rotateAll merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, rotationCenter)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön map'iyle uygular; özel merkezi rotate çağrılarına iletir ve Java `EnumMap` sırasını korur. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 604 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **604/604** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön map'ini, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- NORTH yönünün başlangıç `IDENTITY` dönüşüyle özel merkezi kullanmadan özgün shape'i koruması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1058/1058** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ee01fb100a4d138a2dc2967dc19d79f7e79a055fec252d406d23599850925ae3`.

Birleşik referans kimliği: `b251a60111607ad28c03f7396010655e88be4532d7dc3f048d4f3db054fce03e`.

Yerel raporlar: `compare-v217glqz` (math debug), `compare-snqwxxvj` (math release), `compare-ngv2kroy` (birleşik debug) ve `compare-r5i49tld` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` başlangıç dönüşlü iki argümanlı overload, üç sentetik box ve altı yön map'iyle sınırlıdır. `initial` rotation alan overload, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
