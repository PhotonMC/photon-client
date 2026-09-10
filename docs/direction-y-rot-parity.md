# P01.c — Direction y-rotation Rust davranış kanıtı

`src/direction.rs` içindeki `from_y_rot`, Java `Direction.fromYRot(double)` hesabının bölme, `+ 0.5`, floor, `& 3` ve 2D lookup sırasını taşır. Rust adapter `direction_y_rot` JSONL operation'ı üzerinden bu yöntemi çağırır; Java oracle doğrudan `Direction.fromYRot` kullanır.

Senaryolar eşik açılarını, negatif değerleri, tekrar eden tam turları, signed zero/subnormal girdilerini ve özel double değerlerini kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `a346f61e33f8502c48ca81dee6ee5c27df2c177dc9e95443316cafa4187378ba`, birleşik capture kimliği `ffca09f72f4f265ba069d418009ef267c2669772a52f3ea107cf2037d7a93194` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.fromYRot` sonucunu kapsar; data decoder, axis-direction constructor ve entity bakış akışları ayrı davranış sözleşmeleridir.
