# P01.c — Direction approximate-nearest referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getApproximateNearest(double, double, double)` yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:311-332` incelendi. Double overload her bileşeni f32'ye daraltıp f32 overload'una aktarır; seçim altı yönün normal dot ürünleri üzerinden yapılır.

## Davranış sözleşmesi

- Başlangıç sonucu `north`, en yüksek dot değeri `Float.MIN_VALUE` değeridir.
- Direction enum sırası down, up, north, south, west, east'tir; yalnızca strict büyük dot sonucu mevcut yönü değiştirir.
- Double girdiler Java f32 dönüşümüyle değerlendirilir; zero, tie, Infinity, NaN, subnormal ve taşma sonuç seçiminden önce bu dönüşümden geçer.
- Vec3 overload'u ve float doğrudan overload'u bu dilimin dışındadır; double bileşen yolu ölçülür.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_approximate_nearest"` ve raw f64 bitlerini taşıyan `x`, `y`, `z` alanlarını içerir. Çıktı seçilen yön snapshot'ını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Direction matrix rotation, entity bakış yönü, Vec3 overload'u ve diğer Direction yardımcıları ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Direction::approximate_nearest`, Java double overload'un f32 daraltma ve enum sıralı dot seçim davranışını uygular. Rust adapter `direction_approximate_nearest` JSONL operation'ı ile yön snapshot'ını dışarı verir; Java oracle double overload'u doğrudan çağırır.

Senaryolar yön baskınlığını, tie sırasını, başlangıç eşiğini ve özel f64 girdilerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `52e03eb88f755efc30568fd5a8bee10034fe616f8356b454ad5ac2638c086f02`, birleşik capture kimliği `4616297883a5ce3895e7e1ae687ca0c3e3399de5c30814c5bf72aa8e97f29b07` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getApproximateNearest(double, double, double)` yolunu kapsar; diğer overload'lar ve matrix rotation ayrı davranış sözleşmeleridir.
