# P01.d — Shapes.rotate merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate(shape, rotation, rotationPoint)` yolunu özel merkez bileşenlerinin eksen permütasyonuna, terslenen koordinat listelerinin yansıtılmasına ve discrete hücre dönüşümüne taşıyarak uygular. `examples/behavior_adapter.rs` dört JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 571 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **571/571** math senaryosunda eşleşti.
- Farklı merkez bileşenleriyle Y, X ve Z dönüşleri; koordinat sırasını, merkez çevirisini, yansıtmayı ve shape snapshot'ını karşılaştırdı.
- Identity dönüşünde özel merkezin okunmadan özgün shape'i döndürmesi ayrıca karşılaştırıldı.
- Birleşik altı aile corpus'u **1025/1025** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `bf8eac3fe88ccc7eac0052e8b44cb5dab3dfb456982cd6a700ad1f08fadc02a2`.

Birleşik referans kimliği: `3133f0b82eddafd4383cd19cad3d4e53db2b9051614cfc78968ee4d6d98810b8`.

Yerel raporlar: `compare-wbo29fbq` (math debug), `compare-7m5csdkl` (math release), `compare-tyoie69t` (birleşik debug) ve `compare-_lv2ppe_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik box girdisi ve dört dönüş çağrısıyla sınırlıdır. `CubeVoxelShape` merkez optimizasyonu özel merkezde gözlenebilir biçimde devre dışı bırakılır; diğer shape kaynakları, başlangıç dönüşü, `rotateAttachFace` ve world face/collider toplama sonraki dilimlerdir.
