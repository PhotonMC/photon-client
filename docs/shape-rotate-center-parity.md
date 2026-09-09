# P01.d — Shapes.rotate merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate(shape, rotation, rotationPoint)` yolunu özel merkez bileşenlerinin eksen permütasyonuna, terslenen koordinat listelerinin yansıtılmasına ve discrete hücre dönüşümüne taşıyarak uygular. `examples/behavior_adapter.rs` dört JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 577 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **577/577** math senaryosunda eşleşti.
- Farklı merkez bileşenleriyle Y, X ve Z dönüşleri; koordinat sırasını, merkez çevirisini, yansıtmayı ve shape snapshot'ını karşılaştırdı.
- Identity dönüşünde özel merkezin okunmadan özgün shape'i döndürmesi ayrıca karşılaştırıldı.
- Birleşik altı aile corpus'u **1031/1031** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5df31dedf41786bcd5bb38fc117404df6a83da03bce464c1a637063e5b3b5cbb`.

Birleşik referans kimliği: `cc0f402235532cc834549c53fd069f064a8e771ef6ba6e2943e472b1d9b6b06d`.

Yerel raporlar: `compare-jrhf0iba` (math debug), `compare-8berq09t` (math release), `compare-rpvlapca` (birleşik debug) ve `compare-dwzkalq0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik box girdisi ve dört dönüş çağrısıyla sınırlıdır. `CubeVoxelShape` merkez optimizasyonu özel merkezde gözlenebilir biçimde devre dışı bırakılır; diğer shape kaynakları, başlangıç dönüşü, `rotateAttachFace` ve world face/collider toplama sonraki dilimlerdir.
