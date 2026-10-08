# P01.c — Direction random referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki Direction, Axis ve Plane random seçicilerini ölçer.

## Kaynak ve sınır

Java `core/Direction.java:307-308,519-520,588-594` ve `util/Util.java:788-790` incelendi. Her seçici, kendi sabit dizi uzunluğunu `RandomSource.nextInt` sınırı olarak kullanır ve o indeksin elemanını döndürür.

## Davranış sözleşmesi

- `Direction.getRandom` enum sırasındaki altı yön arasından seçer.
- `Axis.getRandom` X, Y, Z enum sırasındaki üç eksenden seçer.
- `Plane.getRandomDirection`, yatayda north/east/south/west, dikeyde up/down dizisini kullanır.
- `Plane.getRandomAxis`, yatayda x/z, dikeyde y dizisini kullanır.
- Her çağrı bir bounded integer çekişi tüketir; çağrı sırası sonraki random değerini etkiler.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_random"`, signed `seed` ve `plane` (`horizontal` veya `vertical`) alanlarını taşır. Çıktı dört seçimin yön/eksen sonuçlarını ve çekiş sırasını gözlemek için sonraki `tail` i32 değerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Plane shuffle, genel random yardımcı overload'ları, boş liste güvenli seçimleri ve Direction'ın diğer sorguları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki Direction, Axis ve Plane random seçicileri Java'nın `Util.getRandom` bounded-index yolunu uygular. Rust adapter `direction_random` JSONL operation'ı ile aynı `RandomSource` üzerinden dört seçimi ve takip eden çekişi dışarı verir; Java oracle seçicileri doğrudan çağırır.

Senaryolar signed seed sınırlarını, yatay/dikey Plane dizilerini, seçim sırasını ve takip eden random değerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `89a0a5dbabeb0f5eb8d0da566726d2d92e8c28b25b14d755c392bf34ec324ee3`, birleşik capture kimliği `0ea56724b0d8c9168efe02b2ee97f598b4d05649f94f1e5076cc1d15baac2683` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction/Axis/Plane random seçicilerini kapsar; shuffle ve daha geniş random tüketim sıraları ayrı davranış sözleşmeleridir.
