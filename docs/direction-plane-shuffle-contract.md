# P01.c — Direction plane-shuffle referans sözleşmesi

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
