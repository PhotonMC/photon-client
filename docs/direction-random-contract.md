# P01.c — Direction random referans sözleşmesi

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
