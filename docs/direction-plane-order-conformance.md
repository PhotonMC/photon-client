# P01.c — Direction plane-order referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki Plane yüz/eksen dizileri ve `length`, Java iterator, stream ve uzunluk davranışını uygular. Rust adapter `direction_plane_order` JSONL operation'ı ile sıraları dışarı verir; Java oracle iterable, stream ve length yollarını doğrudan çağırır.

Senaryolar HORIZONTAL ve VERTICAL Plane değerlerinin yüz, stream, eksen sıralarını ve uzunluklarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `b85b031110bbbc245b275effa8645808b7383bc5e5b9bc37a97b7b7fdb8402b4`, birleşik capture kimliği `28c7d7162bc0c88d888726d0d0232ec1bbce95c7569973e0cb1dad219d4afe4a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane sıralama ve length yollarını kapsar; random ve shuffle tüketimleri ayrı davranış sözleşmeleridir.
