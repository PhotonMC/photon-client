# P01.c — Direction all-shuffle referans sözleşmesi

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
