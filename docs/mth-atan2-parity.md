# P01.a — `Mth.atan2(double, double)` Rust davranış kanıtı

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
