# P01.c — Direction plane-test Rust davranış kanıtı

`src/direction.rs` içindeki `Plane.test`, Java nullable Direction plane predicate'ini uygular. Rust adapter `direction_plane_test` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı enum predicate'ini doğrudan çağırır.

Senaryolar yatay ve dikey plane değerleri için null girdiyi ve altı yönle eşleşme/eşleşmeme sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `063f63b7eaf27dfcd80efd998335e35bd83da8b16a66c40901e638c86fa1cef4`, birleşik capture kimliği `21018a9765fcbec1c152dfab2d535ae730b86874e06bd452d03fc0ec77b556d2` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane.test predicate'ini kapsar; iterator/stream, random, shuffle ve length yolları ayrı davranış sözleşmeleridir.
