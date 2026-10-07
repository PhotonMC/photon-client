# P01.c — Direction plane-test referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Plane.test`, Java nullable Direction plane predicate'ini uygular. Rust adapter `direction_plane_test` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı enum predicate'ini doğrudan çağırır.

Senaryolar yatay ve dikey plane değerleri için null girdiyi ve altı yönle eşleşme/eşleşmeme sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `063f63b7eaf27dfcd80efd998335e35bd83da8b16a66c40901e638c86fa1cef4`, birleşik capture kimliği `21018a9765fcbec1c152dfab2d535ae730b86874e06bd452d03fc0ec77b556d2` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane.test predicate'ini kapsar; iterator/stream, random, shuffle ve length yolları ayrı davranış sözleşmeleridir.
