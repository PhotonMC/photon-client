# P01.c — Direction axis-test referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.test(Direction)` predicate yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:523-525` incelendi. Predicate, nullable Direction girdisini kabul eder; null durumda false, aksi halde Direction'ın ekseni ile çağrılan Axis kimliğini karşılaştırır.

## Davranış sözleşmesi

- X ekseni yalnızca east ve west yönlerinde true döndürür.
- Y ekseni yalnızca up ve down yönlerinde true döndürür.
- Z ekseni yalnızca north ve south yönlerinde true döndürür.
- null girdide her eksen false döndürür.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_test"`, `axis` ve `direction` alanlarını taşır. `direction` altı yön adından veya nullable girdiyi temsil eden `none` değerinden biridir. Çıktı `value` boolean'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Axis choose overload'ları, plane, random seçim ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.
