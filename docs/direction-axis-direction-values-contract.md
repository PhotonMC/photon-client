# P01.c — Direction axis-direction-values referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.AxisDirection.getStep()` ve `opposite()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:558-573` incelendi. Enum değerleri sabit signed step ve ad alanı taşır; `opposite()` iki enum değeri arasında doğrudan geçiş yapar.

## Davranış sözleşmesi

- `POSITIVE.getStep()` `1`, `NEGATIVE.getStep()` `-1` döndürür.
- `POSITIVE.getName()` ve `NEGATIVE.getName()` sırasıyla `Towards positive` ve `Towards negative` metinleridir.
- `opposite()` pozitif değeri negatife, negatif değeri pozitife çevirir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_direction_values"` ve `direction` (`positive` veya `negative`) alanlarını taşır. Çıktı `step`, `name` ve karşıt değerin `opposite` adını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Axis yön eşlemeleri, predicate'ler, plane, random seçim, coordinate choose overload'ları ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.
