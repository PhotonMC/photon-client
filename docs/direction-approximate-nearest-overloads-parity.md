# P01.c — Direction approximate-nearest overload Rust davranış kanıtı

`src/direction.rs` içindeki `approximate_nearest_f32`, `approximate_nearest` ve `approximate_nearest_vec3`, Java üç overload'un giriş dönüşümlerini ve ortak dot seçim sırasını uygular. Rust adapter `direction_approximate_nearest_overloads` JSONL operation'ı ile üç yön sonucunu dışarı verir; Java oracle üç overload'u doğrudan çağırır.

Senaryolar f32/f64 raw bit girişlerini, forwarding'i, tie sırasını ve özel değerleri Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `891528482167c8462ca952a6e9c61c304c28c90583f98649de3849f7e7de8417`, birleşik capture kimliği `3d4e1cc7790d2d002a7c0787a50d44ed78939a011b198e5cec9c82f6b544e541` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca üç `getApproximateNearest` overload'unu kapsar; matrix rotation ve entity bakış yolları ayrı davranış sözleşmeleridir.
