# P01.a — `Mth.invSqrt` Rust davranış kanıtı

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
