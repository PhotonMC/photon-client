# P01.c — Direction data decoder referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.from3DDataValue(int)` ve `from2DDataValue(int)` static yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:279-289` ve enum başlatma sırasındaki data tabloları incelendi. 3D tablo data3d sırasındaki altı yönü; 2D tablo yalnızca yatay yönleri data2d sırasındaki dört yönü taşır.

## Davranış sözleşmesi

- 3D yol indeksi `abs(data % 6)` ile seçer: `0..5` sırası `down`, `up`, `north`, `south`, `west`, `east`.
- 2D yol indeksi `abs(data % 4)` ile seçer: `0..3` sırası `south`, `west`, `north`, `east`.
- `%` Java signed i32 remainder'dır; negatif input sonrasında `abs` uygulanır. Bu yüzden negatif data genel pozitif modulo gibi yorumlanmaz.
- i32 uçları ve tekrar eden değerler aynı enum snapshot'ı içinde gözlenir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_data"`, `type` (`3d` veya `2d`) ve signed i32 `value` alanlarını içerir. Çıktı yön adı, data değerleri, eksen, karşıt yön ve step bileşenlerini snapshot'lar.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Mevcut capture'lar değiştirilmez; kaynak, girdi veya ortam değiştiğinde yeni referans kimliği oluşur.

## Kapsam dışı

`fromYRot`, `getNearest`, random direction seçimi, codec yolları ve entity bakışına bağlı Direction yardımcıları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `from_3d_data_value` ve `from_2d_data_value`, Java Direction tablolarının kaynak sırasını ve `abs(data % length)` indeksleme davranışını uygular. Rust adapter bu static yolları `direction_data` JSONL operation'ına bağlar; Java oracle doğrudan ilgili enum metodunu çağırır.

Senaryolar pozitif/negatif data değerlerini, tekrar eden remainder'ları ve i32 sınırlarını kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `fff21a34c9aa8e6e49b50373ad3927645acc11494bc864d7a49f2c4cf88cf1db`, birleşik capture kimliği `96910e19049ef799628d62f81c4acc2b45fec7a85324004bb08ec56cef0ae7a8` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca iki Direction data decoder'ının sonucunu kapsar; kalan yön bulma ve entity bakış akışları ayrı davranış sözleşmeleridir.
