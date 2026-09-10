# P01.c — Direction axis-plane referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.getPlane()` yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:527-531` incelendi. Axis enumu X/Z için yatay, Y için dikey Plane değerini döndürür.

## Davranış sözleşmesi

- X ekseni `Plane.HORIZONTAL` sonucunu verir.
- Z ekseni `Plane.HORIZONTAL` sonucunu verir.
- Y ekseni `Plane.VERTICAL` sonucunu verir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_plane"` ve `axis` (`x`, `y` veya `z`) alanlarını taşır. Çıktı `plane` alanında `horizontal` veya `vertical` değerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Plane predicate, random direction/axis, iterator/stream, shuffle ve length yolları ile Axis'in diğer sorguları bu dilime dahil değildir.
