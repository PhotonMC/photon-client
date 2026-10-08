# P01.c — Direction all-shuffle referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.allShuffled(RandomSource)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:126-128` ve `util/Util.java:1060-1078` incelendi. Yöntem Direction enum değerlerinin yeni bir kopyasını oluşturur ve ortak liste shuffle yordamına verir.

## Davranış sözleşmesi

- Başlangıç enum sırası down, up, north, south, west, east'tir.
- Her `i` konumunda sondan başa ilerlenir, `random.nextInt(i)` çekilir ve `i - 1` ile seçilen indeks yer değiştirir.
- Enum değerlerinin kaynak sırası değiştirilmez; yalnızca döndürülen kopya karıştırılır.
- Karıştırma sonrası random kaynağından yapılan değer, çekiş sırasını gözlemlemek için kullanılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_all_shuffle"` ve signed `seed` alanını taşır. Çıktı karıştırılmış `faces` dizisini ve sonraki random i32 `tail` değerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Plane yüz shuffle, genel liste shuffle overload'ları, stream tabanlı `toShuffledList` ve diğer Direction yolları ayrı davranış sözleşmeleridir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `Direction::all_shuffled`, Java `Direction.allShuffled(RandomSource)` enum kopyası ve `Util.shuffledCopy` swap sırasını uygular. Rust adapter `direction_all_shuffle` JSONL operation'ı ile yüzleri ve sonraki random çekişini dışarı verir; Java oracle static metodu doğrudan çağırır.

Senaryolar altı yön enum sırasını, signed seed sınırlarını, shuffle sonucunu ve kaynak tüketimini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `75b43f0882bc1a9c34d9bed8d9bcf99785c461c6b4717d9de8e80747cf2ebe15`, birleşik capture kimliği `930aee0cff62b62fced7458134214a4fb5ef226f8f369a16cf0ce770cde33614` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.allShuffled` yolunu kapsar; Plane ve genel liste shuffle yolları ayrı davranış sözleşmeleridir.
