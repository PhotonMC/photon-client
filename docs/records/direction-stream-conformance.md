# P01.c — Direction stream referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.stream()` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:130-132` incelendi. Yöntem statik Direction değerleri dizisi üzerinden stream oluşturur.

## Davranış sözleşmesi

- Çıktı enum bildirim sırasını korur: down, up, north, south, west, east.
- Her çağrı aynı statik değer sırasını gözlemler; değerler kopyalanıp değiştirilmez.
- Rust iterator çıktısı Java stream eleman adlarıyla karşılaştırılır.

## JSONL arayüzü

İstek `family: "math"` ve `operation: "direction_stream"` alanlarını taşır. Çıktı sıralı `faces` ad dizisini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`Direction.allShuffled`, Plane iterator/stream yolları ve diğer Direction yardımcıları ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Direction::stream`, Java `Direction.stream()` statik değer sırasını iterator olarak sunar. Rust adapter `direction_stream` JSONL operation'ı ile yön adlarını dışarı verir; Java oracle stream'i doğrudan tüketir.

Beklenen sıra Java oracle çıktısından alındı.

Java math capture kimliği `a59b8563452b98bdf7584a0293b26ee8d66f8662512b0f7e94be58b255d65b6c`, birleşik capture kimliği `5517ab5d01567b7143c92dcee7b6ef8d46536afeeabe8e8e7aab81ae78b9ff4d` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.stream` sırasını kapsar; shuffle ve Plane collection yolları ayrı davranış sözleşmeleridir.
