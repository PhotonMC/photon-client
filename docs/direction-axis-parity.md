# P01.c — Direction axis-direction Rust davranış kanıtı

`src/direction.rs` içindeki `from_axis_and_direction`, Java `Direction.fromAxisAndDirection` eksen ve işaret eşlemesini uygular. Rust adapter `direction_axis` JSONL operation'ı üzerinden yöntemi çağırır; Java oracle aynı enum metodunu doğrudan kullanır.

Senaryolar X/Y/Z eksenlerinin pozitif ve negatif AxisDirection çiftlerini kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `a0fcce8574ed7e7504a909bf5d44782eb6188d8f866dbe5eb10e4a567cb0d772`, birleşik capture kimliği `e90cc09edae2a718af616d6ac887e78417d48304d0f8185161e41d82d8e17d85` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-direction constructor sonucunu kapsar; kalan Direction bulma ve dönüşüm akışları ayrı davranış sözleşmeleridir.
