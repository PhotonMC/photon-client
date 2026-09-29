# P01.d — Shapes.rotate merkez overload referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **4 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, bir shape'i verilen `OctahedralGroup` dönüşü ve özel `Vec3 rotationPoint` ile döndüren Java overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:323-372`, `world/phys/shapes/DiscreteVoxelShape.java:24-45` ve `com/mojang/math/OctahedralGroup.java` incelendi. `MathOracle` doğrudan `Shapes.rotate(shape, rotation, rotationPoint)` çağrısını yapar; beklenen shape kutuları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotationPoint` overload'ını kapsar. Varsayılan blok merkezli dönüşler ve bunları kullanan axis/direction map yardımcıları ayrı birleşik kayıtlarda tutulur.

## Davranış sözleşmesi

- Identity dönüşünde Java merkezi okumadan özgün shape'i döndürür; özel merkez identity için gözlenebilir bir fark oluşturmaz.
- Identity dışı dönüşte discrete hücrelerin eksen permütasyonu ve terslenmesi dönüş grubuna göre yapılır. Koordinat düzlemleri, output ekseninin kaynak düzlemindeki `rotationPoint` bileşeninden output ekseninin bileşenine taşınır.
- Terslenen eksende koordinat listesi tersten okunur ve her değer kaynak merkez etrafında yansıtılır; terslenmeyen eksende sıra korunur ve merkez farkı eklenir. Hücre indisleri koordinat dönüşüyle aynı eksen permütasyonu ve tersleme sırasını izler.
- Box shape'in blok merkezli özel dönüş optimizasyonu özel merkezde kullanılmaz; sonuç snapshot'ı `empty`, `boxes` ve varsa `bounds` alanlarıyla gözlenir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Y dönüşü | 1 | Asimetrik box ve eksen bileşenleri farklı merkez |
| X dönüşü | 1 | Unit cube ve özel merkez |
| Z dönüşü | 1 | Y ekseni yarım grid ve özel merkez |
| Identity | 1 | Özel merkez verilmesine rağmen değişmeyen shape |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_center"`, `rotation`, `box` ve `center` alanlarını kullanır. Sonuç tek shape snapshot'ıdır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java 26.2 `Shapes.rotate(shape, rotation, rotationPoint)` yolunu özel merkez bileşenlerinin eksen permütasyonuna, terslenen koordinat listelerinin yansıtılmasına ve discrete hücre dönüşümüne taşıyarak uygular. `examples/behavior_adapter.rs` dört JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Farklı merkez bileşenleriyle Y, X ve Z dönüşleri; koordinat sırasını, merkez çevirisini, yansıtmayı ve shape snapshot'ını karşılaştırdı.
- Identity dönüşünde özel merkezin okunmadan özgün shape'i döndürmesi ayrıca karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört sentetik box girdisi ve dört dönüş çağrısıyla sınırlıdır. `CubeVoxelShape` merkez optimizasyonu özel merkezde gözlenebilir biçimde devre dışı bırakılır; diğer shape kaynakları, başlangıç dönüşü, `rotateAttachFace` ve world face/collider toplama sonraki dilimlerdir.
