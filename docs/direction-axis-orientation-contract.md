# P01.c — Direction axis-orientation referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.isVertical()` ve `isHorizontal()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:498-504` incelendi. Predicate'ler yalnızca enum kimliğini karşılaştırır; koordinat veya yön normalinden hesap yapmaz.

## Davranış sözleşmesi

- `Y.isVertical()` true, `X` ve `Z` false döndürür.
- `X.isHorizontal()` ve `Z.isHorizontal()` true, `Y` false döndürür.
- Her geçerli eksen için iki predicate birbirinin tamamlayıcısıdır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_orientation"` ve `axis` (`x`, `y` veya `z`) alanlarını taşır. Çıktı `vertical` ve `horizontal` boolean alanlarını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Eksenin yön seçimi, `getPlane`, random seçim, coordinate choose overload'ları ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.
