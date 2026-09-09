# P01.d — Shapes.rotateAllAxis merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis(north, rotationCenter)` yolunu X, Y ve Z eksen map'iyle uygular; özel merkezi X/Y dönüşlerine iletir, Z girdisini özgün shape olarak korur ve Java `EnumMap` sırasını taşır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1193 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1193/1193** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, özel merkez çevirisini, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Z yönünün özgün shape'i ve merkezden bağımsız kalması da karşılaştırıldı.
- Birleşik altı aile corpus'u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c`.

Birleşik referans kimliği: `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c`.

Yerel raporlar: `compare-dgq7mp_7` (math debug), `compare-2hmy86uj` (math release), `compare-06i9lguy` (birleşik debug) ve `compare-r6guli2r` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `IDENTITY` yolundaki iki argümanlı overload, üç sentetik box ve X/Y/Z map'iyle sınırlıdır. `rotateHorizontalAxis` merkez overload'ı, başlangıç dönüşü ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
