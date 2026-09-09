# P01.d — Shapes.rotate merkez overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate(shape, rotation, rotationPoint)` yolunu özel merkez bileşenlerinin eksen permütasyonuna, terslenen koordinat listelerinin yansıtılmasına ve discrete hücre dönüşümüne taşıyarak uygular. `examples/behavior_adapter.rs` dört JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1046 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1046/1046** math senaryosunda eşleşti.
- Farklı merkez bileşenleriyle Y, X ve Z dönüşleri; koordinat sırasını, merkez çevirisini, yansıtmayı ve shape snapshot'ını karşılaştırdı.
- Identity dönüşünde özel merkezin okunmadan özgün shape'i döndürmesi ayrıca karşılaştırıldı.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Birleşik referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Yerel raporlar: `compare-m8v98wom` (math debug), `compare-4i19c_i3` (math release), `compare-p7m9abjy` (birleşik debug) ve `compare-vvlbkmvf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik box girdisi ve dört dönüş çağrısıyla sınırlıdır. `CubeVoxelShape` merkez optimizasyonu özel merkezde gözlenebilir biçimde devre dışı bırakılır; diğer shape kaynakları, başlangıç dönüşü, `rotateAttachFace` ve world face/collider toplama sonraki dilimlerdir.
