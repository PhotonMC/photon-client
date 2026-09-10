# P01.c — Direction approximate-nearest overload referans sözleşmesi

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
