# P01.c — Direction unit-vectors Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::unit_vec3` ve `unit_vec3f`, Java Direction normalinin f64 ve f32 görünümlerini sunar; adapter integer görünümü de aynı normal ile birlikte dışarı verir. Java oracle `getUnitVec3i`, `getUnitVec3` ve `getUnitVec3f` yollarını doğrudan çağırır.

Altı yönün üç normal vektör görünümü Java oracle çıktısıyla karşılaştırılır.

Java math capture kimliği `23f29772f5d665ef30bd3d96338ec15e6175df86c7ef81e7cf0419b637d0ec03`, birleşik capture kimliği `9f50bdd395df173070444a5383a3cfcd38805cd0c6f1648535b4fe46c933348d` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction normal vektör getter'larını kapsar; relative, matrix rotation ve entity bakış yolları ayrı davranış sözleşmeleridir.
