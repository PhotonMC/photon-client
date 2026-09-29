# P01.c — Direction axis-directions referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Axis.get_positive`, `get_negative` ve `get_directions`, Java eksen/yön eşlemelerini ve pozitif/negatif dizi sırasını uygular. Rust adapter `direction_axis_directions` JSONL operation'ı ile yön snapshot'larını dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar X, Y ve Z eksenlerinin tekil pozitif/negatif yönlerini ve birleşik dizi sırasını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `4f7b352046e51ad873047d8c1b31b450095876f829ccbaf3482743512da2c481`, birleşik capture kimliği `38ffe865bdd9dd0cb6ae9a43c2001e20f39cee76a5ec14f4911b3bfecbeaa43e` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-directions eşlemelerini kapsar; predicate, plane, random ve coordinate seçimleri ayrı davranış sözleşmeleridir.
