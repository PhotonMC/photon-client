# P01.d — Shapes.rotateHorizontalAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` yolunu X ve Z eksen map'iyle uygular; özel merkezi X dönüşüne iletir, Z girişini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1534 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1534/1534** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug) ve `compare-t1bsi_04` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Z map'iyle sınırlıdır. `rotateAllAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
