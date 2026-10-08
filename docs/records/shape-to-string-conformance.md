# P01.d — VoxelShape toString sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.toString()` sonucunun empty kısa adı ile dolu shape bounds biçimini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:318-321` ve bounds içindeki `world/phys/AABB.java:433-436` yolları incelendi. `MathOracle`, shape'leri Java üreticileriyle oluşturup doğrudan `toString()` sonucunu kaydeder.

## Davranış sözleşmesi

- Boş shape tam olarak `EMPTY` metnini döndürür.
- Dolu shape `VoxelShape[AABB[minX, minY, minZ] -> [maxX, maxY, maxZ]]` biçimini kullanır.
- Bounds endpoint'leri Java `Double.toString` ile yazılır; `.0`, bilimsel gösterim, `-0.0`, `Infinity`, `-Infinity` ve `NaN` sonuçta görünür.
- Shape metni geometrik kutuların listesini değil, tek encompassing bounds AABB'sini gösterir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `empty` | Threshold altında bir box | `EMPTY` |
| `cube` | Grid tabanlı bounded shape | Bounds metni |
| `scientific` | Küçük/büyük exponent, signed zero ve normal değerler | Java double yazım biçimi |
| `infinity` | Sonsuz endpoint'li shape | Infinity metni |
| `nan` | NaN endpoint'li shape | NaN metni |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "to_string"` ve `shape` alanını kullanır.

## Rust karşılığı

`VoxelShape::to_string_java`, empty kısa yolunu, bounds AABB biçimini ve Java'nın özel double metinlerini üretir. Bilimsel gösterimde Rust'ın kısa round-trip rakamları Java'nın `E`/threshold biçimine uyarlanır; trailing zero'lar sonuçta tutulmaz.


## Client-parity doğrulaması

`crates/photon-core/src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
