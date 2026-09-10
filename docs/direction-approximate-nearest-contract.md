# P01.c — Direction approximate-nearest referans sözleşmesi

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
