# P01.d — Shapes.rotateAllAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis(north, rotationCenter)` yolunu X, Y ve Z eksen map'iyle uygular; özel merkezi X/Y dönüşlerine iletir, Z girdisini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 710 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **710/710** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1164/1164** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80`.

Birleşik referans kimliği: `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80`.

Yerel raporlar: `compare-ioen6ccd` (math debug), `compare-r587dixs` (math release), `compare-s2nusmi5` (birleşik debug) ve `compare-2zqsf1m9` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Y/Z map'iyle sınırlıdır. `rotateHorizontalAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
