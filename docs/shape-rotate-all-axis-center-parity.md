# P01.d — Shapes.rotateAllAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis(north, rotationCenter)` yolunu X, Y ve Z eksen map'iyle uygular; özel merkezi X/Y dönüşlerine iletir, Z girdisini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 604 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **604/604** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1058/1058** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ee01fb100a4d138a2dc2967dc19d79f7e79a055fec252d406d23599850925ae3`.

Birleşik referans kimliği: `b251a60111607ad28c03f7396010655e88be4532d7dc3f048d4f3db054fce03e`.

Yerel raporlar: `compare-v217glqz` (math debug), `compare-snqwxxvj` (math release), `compare-ngv2kroy` (birleşik debug) ve `compare-r5i49tld` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Y/Z map'iyle sınırlıdır. `rotateHorizontalAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
