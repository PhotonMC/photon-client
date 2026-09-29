# P01.c — Direction approximate-nearest overload referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getApproximateNearest(float, float, float)`, double overload'u ve `Vec3` overload'unu birlikte ölçer.

## Kaynak ve sınır

Java `core/Direction.java:311-332` incelendi. Float overload f32 bileşenleri doğrudan dot seçim döngüsüne verir; double overload bileşenleri f32'ye daraltır; Vec3 overload'u bileşenleri double yoluna yönlendirir.

## Davranış sözleşmesi

- Üç overload aynı down, up, north, south, west, east enum sırasını, `Float.MIN_VALUE` başlangıç eşiğini ve strict `>` karşılaştırmasını kullanır.
- Float girdiler f32 raw bitleriyle, double ve Vec3 girdileri f64 raw bitleriyle bağımsız olarak sağlanır.
- `Vec3` sonucu double overload ile aynı olmalıdır; f32 ve double girişleri aynı sayısal değeri temsil ettiğinde üç sonuç da aynı yönü verir.
- NaN, Infinity, signed zero ve double→float precision sınırı doğrudan overload'lar arasında karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_approximate_nearest_overloads"`, `f32_x/y/z` raw f32 bitleri ve `x/y/z` raw f64 bitleri alanlarını içerir. Çıktı `f32`, `f64` ve `vec3` yön snapshot'larını taşır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Direction matrix rotation, entity bakış yönü ve diğer Direction yardımcıları ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `approximate_nearest_f32`, `approximate_nearest` ve `approximate_nearest_vec3`, Java üç overload'un giriş dönüşümlerini ve ortak dot seçim sırasını uygular. Rust adapter `direction_approximate_nearest_overloads` JSONL operation'ı ile üç yön sonucunu dışarı verir; Java oracle üç overload'u doğrudan çağırır.

Senaryolar f32/f64 raw bit girişlerini, forwarding'i, tie sırasını ve özel değerleri Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `891528482167c8462ca952a6e9c61c304c28c90583f98649de3849f7e7de8417`, birleşik capture kimliği `3d4e1cc7790d2d002a7c0787a50d44ed78939a011b198e5cec9c82f6b544e541` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca üç `getApproximateNearest` overload'unu kapsar; matrix rotation ve entity bakış yolları ayrı davranış sözleşmeleridir.
