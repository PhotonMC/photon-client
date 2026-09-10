# P01.c — Direction plane-order referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Plane` iterator, stream ve `length()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:576-615` incelendi. Plane sabitleri yüzleri ve eksenleri sabit dizilerde tutar; iterator ve stream yüz dizisinin sırasını korur, length yüz sayısını döndürür.

## Davranış sözleşmesi

- `HORIZONTAL` yüz sırası north, east, south, west; eksen sırası x, z'dir.
- `VERTICAL` yüz sırası up, down; eksen sırası y'dir.
- Iterator ve stream aynı yüz sırasını üretir.
- `length()` yüz dizisinin uzunluğunu döndürür.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_plane_order"` ve `plane` (`horizontal` veya `vertical`) alanlarını taşır. Çıktı `faces`, `stream_faces`, `axes` dizilerini ve `length` değerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Plane random yön/eksen, shuffle ve diğer Direction/Axis sorguları bu dilime dahil değildir.
