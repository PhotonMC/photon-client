# P01.c — Direction axis-directions referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.getPositive()`, `getNegative()` ve `getDirections()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:506-512` incelendi. Her eksen pozitif ve negatif bir Direction eşlemesi taşır; dizi yöntemi bu iki sonucu pozitif, sonra negatif sırayla döndürür.

## Davranış sözleşmesi

- X ekseninin pozitif/negatif yönleri sırasıyla east/west'tir.
- Y ekseninin pozitif/negatif yönleri sırasıyla up/down'dır.
- Z ekseninin pozitif/negatif yönleri sırasıyla south/north'tur.
- `getDirections()` her eksen için `[getPositive(), getNegative()]` sırasını üretir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_directions"` ve `axis` (`x`, `y` veya `z`) alanlarını taşır. Çıktı pozitif/negatif yön snapshot'larını ve aynı sıradaki `directions` dizisini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Eksen predicate'leri, plane, random seçim, coordinate choose overload'ları ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.
