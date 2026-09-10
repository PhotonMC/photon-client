# P01.a — Mth public constants sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` sınıfının public scalar ve eksen sabitlerini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/util/Mth.java:21-30` incelendi. Scalar değerler f32 olarak saklanır; `SQRT_OF_TWO`, aynı sınıftaki `sqrt(float)` yolundan oluşur. `Y_AXIS`, `X_AXIS` ve `Z_AXIS`, `Vector3fc` arayüzü üzerinden f32 birim vektörleridir.

## Davranış sözleşmesi

- `PI`, `HALF_PI`, `TWO_PI`, `DEG_TO_RAD` ve `RAD_TO_DEG`, Java `Math.PI` türetmelerinin f32 sonuçlarını taşır.
- `EPSILON` `1.0E-5F` değeridir; `SQRT_OF_TWO` Java `sqrt(2.0F)` sonucudur.
- `Y_AXIS`, `X_AXIS` ve `Z_AXIS`, sırasıyla y, x ve z bileşeninde `1.0F`, diğer bileşenlerde pozitif `0.0F` taşır.
- Sonuçlar raw f32 bitleriyle karşılaştırılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "mth_constants"` ve küçük harfli `name` alanlarını taşır. Scalar sabitlerde sonuç `bits`; eksen sabitlerinde `value` altında `x`, `y`, `z` f32 bitleri bulunur.

Rust karşılığı `src/math.rs` içindeki `MTH_*` sabitleridir. Oracle sabitleri doğrudan Java alanlarından okur; adapter aynı değerleri üretim API'sinden taşır.

## Kapsam dışı

Bu sabitleri kullanan oyun davranışları, Mth private lookup tabloları ve Vector3f matematik işlemleri ayrı dilimlerdir.
