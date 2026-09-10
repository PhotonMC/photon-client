# P01.c — Direction data decoder Rust davranış kanıtı

`src/direction.rs` içindeki `from_3d_data_value` ve `from_2d_data_value`, Java Direction tablolarının kaynak sırasını ve `abs(data % length)` indeksleme davranışını uygular. Rust adapter bu static yolları `direction_data` JSONL operation'ına bağlar; Java oracle doğrudan ilgili enum metodunu çağırır.

Senaryolar pozitif/negatif data değerlerini, tekrar eden remainder'ları ve i32 sınırlarını kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `fff21a34c9aa8e6e49b50373ad3927645acc11494bc864d7a49f2c4cf88cf1db`, birleşik capture kimliği `96910e19049ef799628d62f81c4acc2b45fec7a85324004bb08ec56cef0ae7a8` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca iki Direction data decoder'ının sonucunu kapsar; kalan yön bulma ve entity bakış akışları ayrı davranış sözleşmeleridir.
