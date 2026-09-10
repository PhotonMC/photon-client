# P01.c — Direction y-rotation encoder Rust davranış kanıtı

`src/direction.rs` içindeki `to_y_rot`, Java `Direction.toYRot()` data2d maskesi ve 90 derecelik f32 dönüşümünü uygular. Rust adapter `direction_to_y_rot` JSONL operation'ı üzerinden sonucu raw bitlerle dışarı verir; Java oracle doğrudan enum metodunu çağırır.

Altı yön adı test edildi. Beklenen f32 bitleri Java oracle çıktısından alındı.

Java math capture kimliği `6333b31ddd0c9e44a85eb6b8008b8eac82df5edff51920f8c6c32da5d9b690b4`, birleşik capture kimliği `3db4b23f685b322fabe26e2db418bfb98be927e352d42d87c2953a34a5c3688b` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.toYRot` sonucunu kapsar; `fromYRot` seçimi ve diğer Direction dönüşümleri ayrı davranış sözleşmeleridir.
