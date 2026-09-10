# P01.c — Direction all-shuffle Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::all_shuffled`, Java `Direction.allShuffled(RandomSource)` enum kopyası ve `Util.shuffledCopy` swap sırasını uygular. Rust adapter `direction_all_shuffle` JSONL operation'ı ile yüzleri ve sonraki random çekişini dışarı verir; Java oracle static metodu doğrudan çağırır.

Senaryolar altı yön enum sırasını, signed seed sınırlarını, shuffle sonucunu ve kaynak tüketimini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `75b43f0882bc1a9c34d9bed8d9bcf99785c461c6b4717d9de8e80747cf2ebe15`, birleşik capture kimliği `930aee0cff62b62fced7458134214a4fb5ef226f8f369a16cf0ce770cde33614` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.allShuffled` yolunu kapsar; Plane ve genel liste shuffle yolları ayrı davranış sözleşmeleridir.
