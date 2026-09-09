# P01.b — Vec3 referans sözleşmesi

Bu dilimin **40 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](math-parity.md)). Dot/cross/vectorTo/add/subtract/distanceTo/distanceToSqr, reverse, multiply, horizontal, horizontalDistance, horizontalDistanceSqr, closerThan overload'ları, offsetRandom, offsetRandomXZ, lerp, rotateClockwise90, isFinite, get, with, relative, align, projectedOn, toVector3f ve diğer Vec3 yardımcıları bu dilime dahil değildir. Vec3 koordinat yardımcıları ayrı [sözleşme](vec3-coordinates-contract.md) ve [kanıt](vec3-coordinates-parity.md) ile izlenir. Vec3 equals/hashCode ayrı [sözleşme](vec3-equality-contract.md) ve [kanıt](vec3-equality-parity.md) ile izlenir. Vec3 toString ayrı [sözleşme](vec3-to-string-contract.md) ve [kanıt](vec3-to-string-parity.md) ile izlenir. Add/subtract overload'ları için ayrı [sözleşme](vec3-add-subtract-contract.md) ve [kanıt](vec3-add-subtract-parity.md), vectorTo için ayrı [sözleşme](vec3-vector-to-contract.md) ve [kanıt](vec3-vector-to-parity.md), dot için ayrı [sözleşme](vec3-dot-contract.md) ve [kanıt](vec3-dot-parity.md), cross için ayrı [sözleşme](vec3-cross-contract.md) ve [kanıt](vec3-cross-parity.md), distanceTo için ayrı [sözleşme](vec3-distance-to-contract.md) ve [kanıt](vec3-distance-to-parity.md), distanceToSqr için ayrı [sözleşme](vec3-distance-to-sqr-contract.md) ve [kanıt](vec3-distance-to-sqr-parity.md), reverse için ayrı [sözleşme](vec3-reverse-contract.md) ve [kanıt](vec3-reverse-parity.md), multiply için ayrı [sözleşme](vec3-multiply-contract.md) ve [kanıt](vec3-multiply-parity.md), horizontal için ayrı [sözleşme](vec3-horizontal-contract.md) ve [kanıt](vec3-horizontal-parity.md), horizontalDistance için ayrı [sözleşme](vec3-horizontal-distances-contract.md) ve [kanıt](vec3-horizontal-distances-parity.md), closerThan için ayrı [sözleşme](vec3-closer-than-contract.md) ve [kanıt](vec3-closer-than-parity.md), offsetRandom için ayrı [sözleşme](vec3-offset-random-contract.md) ve [kanıt](vec3-offset-random-parity.md), lerp için ayrı [sözleşme](vec3-lerp-contract.md) ve [kanıt](vec3-lerp-parity.md), rotateClockwise90 için ayrı [sözleşme](vec3-rotate-clockwise-90-contract.md) ve [kanıt](vec3-rotate-clockwise-90-parity.md), isFinite için ayrı [sözleşme](vec3-is-finite-contract.md) ve [kanıt](vec3-is-finite-parity.md), get için ayrı [sözleşme](vec3-get-contract.md) ve [kanıt](vec3-get-parity.md), with için ayrı [sözleşme](vec3-with-contract.md) ve [kanıt](vec3-with-parity.md), relative için ayrı [sözleşme](vec3-relative-contract.md) ve [kanıt](vec3-relative-parity.md), align için ayrı [sözleşme](vec3-align-contract.md) ve [kanıt](vec3-align-parity.md), projectedOn için ayrı [sözleşme](vec3-projected-on-contract.md) ve [kanıt](vec3-projected-on-parity.md), toVector3f için ayrı [sözleşme](vec3-to-vector3f-contract.md) ve [kanıt](vec3-to-vector3f-parity.md) tutulur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `world/phys/Vec3.java` içindeki `normalize`, `scale`, `length`, `lengthSqr`, `reverse`, `multiply`, `horizontal`, `horizontalDistance`, `horizontalDistanceSqr`, `closerThan`, `offsetRandom`, `offsetRandomXZ`, `lerp` ve `rotateClockwise90` yardımcıları ile `isFinite` sorgusu, `get` seçimi, `with` değişimi, `relative` hareketi ve `align`, `projectedOn` ve `toVector3f` işlemleri incelendi. Vec3 koordinat yardımcıları için `Vec3.java:45-63` ve `Vec3i.java:65-75` ayrıca incelendi. Vec3 equals/hashCode için `Vec3.java:196-221`, toString için `Vec3.java:224-226` ayrıca incelendi. Adaptör doğrudan orijinal sınıfı çağırır; oyun algoritması adaptörde yeniden yazılmaz.

`Vec3.<clinit>` Netty buffer istediğinden math ailesine `netty-buffer`/`netty-common` eklendi. Yerel JAR SHA-1 değerleri `tests/behavior/reference.json` içinde sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- Bileşenler f64'tür; ham IEEE-754 bitleriyle karşılaştırılır, tolerans yoktur.
- `lengthSqr` bileşen karelerinin double toplamıdır; `length` bunun kareköküdür.
- `normalize` önce `length` hesaplar; sonuç `1.0E-5F` değerinden küçükse sıfır vektör döner, yoksa her bileşen uzunluğa ayrı bölünür. Eşik float literalin double genişlemesidir.
- `scale` her bileşeni aynı çarpanla çarpar.
- Her senaryo başlangıç ve her adım sonrası `x`, `y`, `z`, `length`, `length_squared` bitlerini kaydeder.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Vec3 normalize | 34 | 27 diyagonal/eksen yönü, işaretli sıfır, subnormal, en büyük sonlu, karışık bileşen ve 1e-5 eşiğinin bit komşuları |
| Vec3 zincir | 6 | Normalize → ölçek → normalize; sıfır/negatif/pozitif çarpanlar |

`family: "math"`, `operation: "vec3"` ortak girdilerdir. `x`, `y`, `z` ve `factor` 16 haneli küçük harfli f64 bit stringleridir. `steps` içindeki `normalize` ve `factor` alanlı `scale` çağrıları sırayla uygulanır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

220 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `9d468b5b5edbbe267fb1a4133bfff61c3e528f274f55cb9decd6738beaca43ed`. Önceki 180 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 650 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `8ca11f8017cf6a605669c6437c7df292092304e47e7b567655dc48b195788b64`. Eski kayıtlar değiştirilmedi.

Mevcut Rust matematik adaptörü Vec3 içerir. Beklenen çıktılar Rust'a göre değiştirilmedi.
