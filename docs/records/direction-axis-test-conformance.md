# P01.c — Direction axis-test referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `Axis.test`, Java nullable Direction predicate'inin eksen eşleşmesini uygular. Rust adapter `direction_axis_test` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı enum predicate'ini doğrudan çağırır.

Senaryolar üç eksenin null girdisini ve altı yönle eşleşme/eşleşmeme sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `7bb705f415ebab47341d086e6112ad49bc6374c12fc2ff0ab7904038da991692`, birleşik capture kimliği `a0d71fc8b3981a371f1cc0f1a6075f8760a9ab4edc6e9e36ae33f9d3e1f9398a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Axis.test predicate'ini kapsar; choose overload'ları, plane, random ve diğer Direction sorguları ayrı davranış sözleşmeleridir.
