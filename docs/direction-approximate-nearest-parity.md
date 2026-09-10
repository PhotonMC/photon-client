# P01.c — Direction approximate-nearest Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::approximate_nearest`, Java double overload'un f32 daraltma ve enum sıralı dot seçim davranışını uygular. Rust adapter `direction_approximate_nearest` JSONL operation'ı ile yön snapshot'ını dışarı verir; Java oracle double overload'u doğrudan çağırır.

Senaryolar yön baskınlığını, tie sırasını, başlangıç eşiğini ve özel f64 girdilerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `52e03eb88f755efc30568fd5a8bee10034fe616f8356b454ad5ac2638c086f02`, birleşik capture kimliği `4616297883a5ce3895e7e1ae687ca0c3e3399de5c30814c5bf72aa8e97f29b07` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getApproximateNearest(double, double, double)` yolunu kapsar; diğer overload'lar ve matrix rotation ayrı davranış sözleşmeleridir.
