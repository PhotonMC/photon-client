# P01.c — Direction plane-shuffle Rust davranış kanıtı

`src/direction.rs` içindeki `Plane.shuffled_copy`, Java `Util.shuffledCopy` kopyalama ve Fisher-Yates swap sırasını uygular. Rust adapter `direction_plane_shuffle` JSONL operation'ı ile kopyalanmış yüzleri ve sonraki random çekişi dışarı verir; Java oracle `Plane.shuffledCopy` yöntemini doğrudan çağırır.

Senaryolar yatay/dikey yüz listelerini, signed seed sınırlarını, swap sırasını ve kaynak tüketimini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `8d6cd1f94c446ed332f386d41bf11b3bf031c477eaebeb01ccd2b886a27264cd`, birleşik capture kimliği `d40e9b4aab564a900160ec1c09ca0d4e509037f7dbff9e5d40001330420744b4` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Plane.shuffledCopy yolunu kapsar; genel liste shuffle ve stream yolları ayrı davranış sözleşmeleridir.
