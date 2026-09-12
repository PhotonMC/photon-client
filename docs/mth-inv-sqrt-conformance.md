# P01.a — `Mth.invSqrt` referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/math.rs` içindeki `mth_inv_sqrt_f32` ve `mth_inv_sqrt_f64`, Java `Mth.invSqrt` ile pinned JOML `Math.invsqrt` yollarını JSONL oracle'a bağlar. Float ve double overload'ları kendi kaynak genişliklerinde karşılaştırılır.

Java oracle doğrudan `Mth.invSqrt` çağrısını yürütür. Senaryolar sıfır, signed zero, sonlu değerler, subnormal, negatif, Infinity ve NaN girdilerini kapsar. Beklenen sonuçlar Rust çıktısından türetilmez.

Java math capture kimliği:

`4b0546a9e73c6a57ee5f0d26f59206ebe7b4b9dd0fa4fb59e4ba5601b93da3ba`

Yeni birleşik capture kimliği:

`f33070aafc389844decb69fa30af030b5198435f0df122e79936036781f64c25`

Capture'lar ve karşılaştırma raporları `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Mth.invSqrt` overload'larının sayısal sonucunu kapsar; gerçek istemci geometri ve hareket çağrıları ayrı davranış sözleşmeleridir.
