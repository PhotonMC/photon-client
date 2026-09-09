# P01.d — Shapes.rotateHorizontalAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` yolunu X ve Z eksen map'iyle uygular; özel merkezi X dönüşüne iletir, Z girişini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 741 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **741/741** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1195/1195** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7eb966aeeec47bb875a064f29ee8e61b18ab59b1678b26c179ead90046713129`.

Birleşik referans kimliği: `7eb966aeeec47bb875a064f29ee8e61b18ab59b1678b26c179ead90046713129`.

Yerel raporlar: `compare-bgegqdoc` (math debug), `compare-lkhuntdt` (math release), `compare-crkt4gor` (birleşik debug) ve `compare-7us5i_vu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Z map'iyle sınırlıdır. `rotateAllAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
