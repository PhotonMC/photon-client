# P01.a — `Mth.atan2(double, double)` referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki Minecraft'a özgü f64 açı yaklaşımını ölçer. Girdiler ve sonuç ham f64 bitleriyle taşınır; sonuç `Math.atan2` için genel kütüphane çağrısı değildir.

## Kaynak ve sınır

Java kaynağı `util/Mth.java:391-439` ve tablo başlatıcısı `771-778` incelendi. `atan2` kendi normalize etme, asin/cos tablo ve düzeltme polinomunu kullanır; tablo 257 girişte `Math.asin(ind / 256.0)` ve bunun kosinüsünü saklar.

## Davranış sözleşmesi

- Önce `x * x + y * y` hesaplanır; sonuç NaN ise canonical Java NaN döner.
- `y < 0.0` ve `x < 0.0` kararları signed zero'yu negatif kabul etmez. Mutlaklaştırma ve `y > x` steep kararı bundan sonra gelir.
- `fastInvSqrt(d2)` ile x/y normalize edilir. Bias eklenmiş y'nin ham double bitlerinden Java int daraltmasıyla tablo indeksi çıkarılır; tablo örneği 1/256 aralıklı asin/cos değeridir.
- Sine farkı, düzeltme polinomu ve `steep`, x işareti, y işareti dönüşleri kaynak işlem sırasıyla uygulanır. Pi değeri yalnızca steep ve x işareti dönüşlerinde kullanılır.
- Zero, signed zero, tablo sınırı, subnormal, sonlu taşma, Infinity ve NaN girdileri ayrı yollar olarak gözlenir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "mth_atan2"`, `y` ve `x` alanlarını içerir. Alanlar on altı haneli raw f64 bitleridir; çıktı `bits` alanında aynı biçimdedir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Mevcut capture'lar değiştirilmez; kaynak, girdi veya ortam değiştiğinde yeni referans kimliği oluşur.

## Kapsam dışı

JOML `invSqrt`, standart Java `Math.atan2`, Vec3 rotation forwarding'i ve istemci kamera/oyuncu açı akışı bu dilimin dışındadır.
