# P01.c — Direction data decoder referans sözleşmesi

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
