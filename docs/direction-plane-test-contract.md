# P01.c — Direction plane-test referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Plane.test(Direction)` predicate yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:596-598` incelendi. Plane predicate, nullable Direction girdisinin ekseninden türeyen Plane değerini kendi enum değeriyle karşılaştırır.

## Davranış sözleşmesi

- `HORIZONTAL.test` north, east, south ve west yönlerinde true döndürür.
- `VERTICAL.test` up ve down yönlerinde true döndürür.
- Her iki plane null girdide false döndürür.
- Bir Direction yalnızca kendi Axis plane'iyle eşleşir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_plane_test"`, `plane` (`horizontal` veya `vertical`) ve `direction` (altı yön adı veya `none`) alanlarını taşır. Çıktı `value` boolean'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Plane iterator/stream, random, shuffle ve length yolları ile Axis'in diğer sorguları bu dilime dahil değildir.
