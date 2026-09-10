# P01.c — Direction stream Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::stream`, Java `Direction.stream()` statik değer sırasını iterator olarak sunar. Rust adapter `direction_stream` JSONL operation'ı ile yön adlarını dışarı verir; Java oracle stream'i doğrudan tüketir.

Beklenen sıra Java oracle çıktısından alındı.

Java math capture kimliği `a59b8563452b98bdf7584a0293b26ee8d66f8662512b0f7e94be58b255d65b6c`, birleşik capture kimliği `5517ab5d01567b7143c92dcee7b6ef8d46536afeeabe8e8e7aab81ae78b9ff4d` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.stream` sırasını kapsar; shuffle ve Plane collection yolları ayrı davranış sözleşmeleridir.
