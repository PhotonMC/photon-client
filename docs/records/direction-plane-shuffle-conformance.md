# P01.c — Direction plane-shuffle referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Plane.shuffledCopy(RandomSource)` yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:576-610` ve `util/Util.java:1060-1078` incelendi. `shuffledCopy`, Plane yüz dizisinin bağımsız kopyasını oluşturur ve Java `shuffle` yordamının sondan başa swap döngüsünü bu kopyaya uygular.

## Davranış sözleşmesi

- Yatay başlangıç sırası north, east, south, west; dikey başlangıç sırası up, down'dır.
- Her `i` konumunda `random.nextInt(i + 1)` çekilir ve seçilen indeks ile `i` yer değiştirir.
- Orijinal Plane yüz sırası korunur; yalnızca döndürülen kopya karıştırılır.
- Karıştırma sonrası random kaynağından yapılan değer, çekiş sayısını ve sırasını gözlemlemek için kullanılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_plane_shuffle"`, signed `seed` ve `plane` (`horizontal` veya `vertical`) alanlarını taşır. Çıktı karıştırılmış `faces` dizisini ve sonraki random i32 `tail` değerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Genel liste shuffle overload'ları, stream tabanlı `toShuffledList`, random seçiciler ve diğer Direction/Plane yolları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `Plane.shuffled_copy`, Java `Util.shuffledCopy` kopyalama ve Fisher-Yates swap sırasını uygular. Rust adapter `direction_plane_shuffle` JSONL operation'ı ile kopyalanmış yüzleri ve sonraki random çekişi dışarı verir; Java oracle `Plane.shuffledCopy` yöntemini doğrudan çağırır.

Senaryolar yatay/dikey yüz listelerini, signed seed sınırlarını, swap sırasını ve kaynak tüketimini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `8d6cd1f94c446ed332f386d41bf11b3bf031c477eaebeb01ccd2b886a27264cd`, birleşik capture kimliği `d40e9b4aab564a900160ec1c09ca0d4e509037f7dbff9e5d40001330420744b4` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane.shuffledCopy yolunu kapsar; genel liste shuffle ve stream yolları ayrı davranış sözleşmeleridir.
