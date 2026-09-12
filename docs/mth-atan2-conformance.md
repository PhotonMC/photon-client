# P01.a — `Mth.atan2(double, double)` referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/math.rs` içindeki `mth_atan2`, Java `Mth.atan2(double, double)` için normalize etme, 257 girişli tablo kuantizasyonu, düzeltme polinomu ve quadrant dönüşlerini aynı sırayla yürütür. Tablo değerleri çalışma zamanında aynı `ind / 256.0`, `asin` ve `cos(asin)` tanımıyla üretilir; özgün Java kodu Rust'a kopyalanmaz.

Java oracle doğrudan `Mth.atan2(y, x)` çağrısını yürütür. Senaryolar signed zero, eksenler, quadrant'lar, steep/table sınırları, subnormal değerler, taşan norm, Infinity ve NaN yollarını içerir. Beklenen sonuçlar Java oracle çıktısından alındı.

Java math capture kimliği:

`83704751118ade4825964fb9cf52bc48b674d6e85d1e976f0ded804354451836`

Yeni birleşik capture kimliği:

`03ad7028fd8e6851e25cf6f79f3abc221bc8376fc6c0428c39189d3b156f32e6`

Capture'lar ve karşılaştırma raporları `.data/` altında tutulur; kimlik kaynak, girdi ve ortamla birlikte runner tarafından üretilir.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Mth yardımcı metodunun f64 sonucunu kapsar; kamera ve Vec3 açı akışı ayrı davranış sözleşmeleridir.
