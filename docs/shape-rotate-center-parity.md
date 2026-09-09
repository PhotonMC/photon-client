# P01.d — Shapes.rotate merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate(shape, rotation, rotationPoint)` yolunu özel merkez bileşenlerinin eksen permütasyonuna, terslenen koordinat listelerinin yansıtılmasına ve discrete hücre dönüşümüne taşıyarak uygular. `examples/behavior_adapter.rs` dört JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 601 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **601/601** math senaryosunda eşleşti.
- Farklı merkez bileşenleriyle Y, X ve Z dönüşleri; koordinat sırasını, merkez çevirisini, yansıtmayı ve shape snapshot'ını karşılaştırdı.
- Identity dönüşünde özel merkezin okunmadan özgün shape'i döndürmesi ayrıca karşılaştırıldı.
- Birleşik altı aile corpus'u **1055/1055** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3`.

Birleşik referans kimliği: `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52`.

Yerel raporlar: `compare-8dsunddp` (math debug), `compare-zvv3a_3_` (math release), `compare-9k3hskia` (birleşik debug) ve `compare-yw9njwwe` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik box girdisi ve dört dönüş çağrısıyla sınırlıdır. `CubeVoxelShape` merkez optimizasyonu özel merkezde gözlenebilir biçimde devre dışı bırakılır; diğer shape kaynakları, başlangıç dönüşü, `rotateAttachFace` ve world face/collider toplama sonraki dilimlerdir.
