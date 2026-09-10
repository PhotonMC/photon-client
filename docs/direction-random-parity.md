# P01.c — Direction random Rust davranış kanıtı

`src/direction.rs` içindeki Direction, Axis ve Plane random seçicileri Java'nın `Util.getRandom` bounded-index yolunu uygular. Rust adapter `direction_random` JSONL operation'ı ile aynı `RandomSource` üzerinden dört seçimi ve takip eden çekişi dışarı verir; Java oracle seçicileri doğrudan çağırır.

Senaryolar signed seed sınırlarını, yatay/dikey Plane dizilerini, seçim sırasını ve takip eden random değerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `89a0a5dbabeb0f5eb8d0da566726d2d92e8c28b25b14d755c392bf34ec324ee3`, birleşik capture kimliği `0ea56724b0d8c9168efe02b2ee97f598b4d05649f94f1e5076cc1d15baac2683` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction/Axis/Plane random seçicilerini kapsar; shuffle ve daha geniş random tüketim sıraları ayrı davranış sözleşmeleridir.
