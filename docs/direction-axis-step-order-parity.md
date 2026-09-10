# P01.c — Direction axis-step-order Rust davranış kanıtı

`src/direction.rs` içindeki `axis_step_order`, Java `Direction.axisStepOrder(Vec3)` strict X/Z mutlak büyüklük karşılaştırmasını ve iki sabit eksen sırasını uygular. Rust adapter `direction_axis_step_order` JSONL operation'ı ile eksen listesini dışarı verir; Java oracle doğrudan Vec3 overload'unu çağırır.

Senaryolar işaretli büyüklükleri, eşitliği, sıfır/subnormal değerleri, sonsuzlukları ve NaN karşılaştırmalarını kapsar. Beklenen eksen dizileri Java oracle çıktısından alındı.

Java math capture kimliği `acde9c1776a4e0d57ac928e921d52cff658b64f5de0022efaf581073a7cf6200`, birleşik capture kimliği `71ba276d06d37c7bf75b45a4b43fb97b815d1ec4cc2152d79165d5425df2c9e0` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-step-order sonucunu kapsar; diğer Direction bulma, dönüştürme ve entity bakış akışları ayrı davranış sözleşmeleridir.
