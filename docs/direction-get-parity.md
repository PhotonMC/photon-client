# P01.c — Direction get Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::get`, Java `Direction.get(AxisDirection, Axis)` lookup'ını parametre sırası korunarak uygular. Rust adapter `direction_get` JSONL operation'ı üzerinden yön snapshot'ını dışarı verir; Java oracle static lookup metodunu doğrudan çağırır.

Altı axis-direction çifti Java oracle çıktısıyla karşılaştırılır.

Java math capture kimliği `96c733cc1ba2e170149c44d674090b3cde045749fc67c8bcf8a2fb09049a7155`, birleşik capture kimliği `0c710be232812e0f45b5412691acde56c06561cf701020ad71949bf4e31a3f0a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.get` lookup yolunu kapsar; `fromAxisAndDirection` ve diğer Direction yolları ayrı davranış sözleşmeleridir.
