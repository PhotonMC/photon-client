# P01.a — `Mth.invSqrt` referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Mth.invSqrt(float)` ve `Mth.invSqrt(double)` yollarını ölçer. Her overload JOML 1.10.8 içindeki `org.joml.Math.invsqrt` yönlendirmesini korur.

## Kaynak ve sınır

Java kaynağı `util/Mth.java:440-447` incelendi. Pinned JOML bytecode'unda float overload'u `Math.sqrt((double) x)` sonucunu f32'ye daraltıp `1.0F` ile böler; double overload'u `1.0 / Math.sqrt(x)` uygular.

## Davranış sözleşmesi

- Float girdisi önce f64'e yükseltilir, `Math.sqrt` sonucu f32'ye daraltılır ve sonrasında f32 bölme yapılır.
- Double girdisi `Math.sqrt` ile aynı f64 genişliğinde işlenir ve `1.0 / root` sonucu döner.
- Signed zero, negatif değerler, subnormal, Infinity ve NaN yollarında Java floating-point sonucu ve raw bitleri korunur.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "mth_inv_sqrt"`, `type` (`f32` veya `f64`) ve `input` alanlarını içerir. Girdi ve çıktı ham bit dizgileridir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Mevcut capture'lar değiştirilmez; kaynak, girdi veya ortam değiştiğinde yeni referans kimliği oluşur.

## Kapsam dışı

JOML'ın başka yaklaşık kök yöntemleri, `Mth.fastInvSqrt`, `Mth.atan2` ve bu helper'ın geometri/oyuncu akışlarındaki tüketim sırası bu dilime dahil değildir.
