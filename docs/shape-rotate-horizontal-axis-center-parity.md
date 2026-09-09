# P01.d — Shapes.rotateHorizontalAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` yolunu X ve Z eksen map'iyle uygular; özel merkezi X dönüşüne iletir, Z girişini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 695 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **695/695** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1149/1149** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Birleşik referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Yerel raporlar: `compare-z24w3ncm` (math debug), `compare-ue0l25nl` (math release), `compare-oiqblje5` (birleşik debug) ve `compare-pl0oao3t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Z map'iyle sınırlıdır. `rotateAllAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
