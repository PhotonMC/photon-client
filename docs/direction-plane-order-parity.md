# P01.c — Direction plane-order Rust davranış kanıtı

`src/direction.rs` içindeki Plane yüz/eksen dizileri ve `length`, Java iterator, stream ve uzunluk davranışını uygular. Rust adapter `direction_plane_order` JSONL operation'ı ile sıraları dışarı verir; Java oracle iterable, stream ve length yollarını doğrudan çağırır.

Senaryolar HORIZONTAL ve VERTICAL Plane değerlerinin yüz, stream, eksen sıralarını ve uzunluklarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `b85b031110bbbc245b275effa8645808b7383bc5e5b9bc37a97b7b7fdb8402b4`, birleşik capture kimliği `28c7d7162bc0c88d888726d0d0232ec1bbce95c7569973e0cb1dad219d4afe4a` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane sıralama ve length yollarını kapsar; random ve shuffle tüketimleri ayrı davranış sözleşmeleridir.
