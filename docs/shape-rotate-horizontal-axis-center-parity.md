# P01.d — Shapes.rotateHorizontalAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` yolunu X ve Z eksen map'iyle uygular; özel merkezi X dönüşüne iletir, Z girişini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2023 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2023/2023** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Yerel raporlar: `compare-4sx1lm2m` (math debug), `compare-m3ib6sjn` (math release), `compare-czvhou3w` (birleşik debug) ve `compare-oz19j96e` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Z map'iyle sınırlıdır. `rotateAllAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
