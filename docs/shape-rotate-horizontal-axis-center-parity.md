# P01.d — Shapes.rotateHorizontalAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` yolunu X ve Z eksen map'iyle uygular; özel merkezi X dönüşüne iletir, Z girişini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 583 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **583/583** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1037/1037** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a68619cf6f1338df37784d434e06d0952b6dbaab0962ece1f5a77fcbda2b03a6`.

Birleşik referans kimliği: `b0cbcb74aa0f40450d03ba7ed23e6155f13776414b6dc51e556a0b02e08fa2c4`.

Yerel raporlar: `compare-qjy5sea_` (math debug), `compare-hv0y0sp6` (math release), `compare-be91aw33` (birleşik debug) ve `compare-h5dd6zge` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Z map'iyle sınırlıdır. `rotateAllAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
