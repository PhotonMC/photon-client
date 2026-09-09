# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 552 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **552/552** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1006/1006** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5161df0d1744dc53eed78d5785d13024594d5b8306cdcf50f765118ae2042450`.

Birleşik referans kimliği: `bbcdd8d522a3e0799b6945441c0c42cd1e0c98bbb8c9dceda370d0203bed11e9`.

Yerel raporlar: `compare-941j34iv` (math debug), `compare-73_p7rtu` (math release), `compare-88gexq6y` (birleşik debug) ve `compare-dn88muge` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. `initial` rotation, merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
