# P01.d — Shapes.rotateAllAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis(north, rotationCenter)` yolunu X, Y ve Z eksen map'iyle uygular; özel merkezi X/Y dönüşlerine iletir, Z girdisini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 660 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **660/660** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1114/1114** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8`.

Birleşik referans kimliği: `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8`.

Yerel raporlar: `compare-3kg8m9ta` (math debug), `compare-6crwfl2x` (math release), `compare-_35t8r3l` (birleşik debug) ve `compare-g5oda9d_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Y/Z map'iyle sınırlıdır. `rotateHorizontalAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
