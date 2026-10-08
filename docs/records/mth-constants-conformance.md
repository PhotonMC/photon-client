# P01.a — Mth public constants sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

Rust karşılığı `crates/photon-core/src/math.rs` içindeki `MTH_*` sabitleridir. Oracle sabitleri doğrudan Java alanlarından okur; adapter aynı değerleri üretim API'sinden taşır.

## Kapsam dışı

Bu sabitleri kullanan oyun davranışları, Mth private lookup tabloları ve Vector3f matematik işlemleri ayrı dilimlerdir.


## Client-parity doğrulaması

Java `Mth` scalar ve eksen sabitleri, Rust `MTH_*` API'siyle raw f32 bitleri üzerinden karşılaştırılır. Java oracle sabit alanlarını doğrudan okur; Rust adapter üretim sabitlerini JSONL'e taşır.

Java 25 / Linux x86_64 referansında scalar bitleri ve üç eksen vektörünün f32 bileşenleri debug ve release Rust adapter'larında eşleşti. Math capture kimliği `2e05c75c05118adf2dd2ca0cdc420fee4e30be9ff3ebb9c3ac0048aecbaf1af7`, birleşik capture kimliği `9c2116e5b40f80a6d1a615d8660ee3ffbc47cacb979b1a7bb998b9ba6cc696ce` olarak, kaynak ve girdilerle birlikte `.data/captures/behavior/` altında tutulur.

Doğrulama komutları:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca public sabit değerlerini kapsar; sabitlerin sonraki istemci ve matematik çağrı zincirlerindeki tüketicileri ayrı davranış dilimleridir.
