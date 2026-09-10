# P01.c — Direction axis-test Rust davranış kanıtı

`src/direction.rs` içindeki `Axis.test`, Java nullable Direction predicate'inin eksen eşleşmesini uygular. Rust adapter `direction_axis_test` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı enum predicate'ini doğrudan çağırır.

Senaryolar üç eksenin null girdisini ve altı yönle eşleşme/eşleşmeme sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `7bb705f415ebab47341d086e6112ad49bc6374c12fc2ff0ab7904038da991692`, birleşik capture kimliği `a0d71fc8b3981a371f1cc0f1a6075f8760a9ab4edc6e9e36ae33f9d3e1f9398a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Axis.test predicate'ini kapsar; choose overload'ları, plane, random ve diğer Direction sorguları ayrı davranış sözleşmeleridir.
