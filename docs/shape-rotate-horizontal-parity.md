# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1407/1407** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a2c69ab98e59ffb82410fd45282bc6e8c110fd13877ba87471c7bfcc29f23318`.

Birleşik referans kimliği: `a2c69ab98e59ffb82410fd45282bc6e8c110fd13877ba87471c7bfcc29f23318`.

Yerel raporlar: `compare-psz_s0y8` (math debug), `compare-xhpjkx1_` (math release), `compare-smgo22vh` (birleşik debug) ve `compare-h2kr9uj0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. Desteklenen initial overload ayrı [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md); merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
