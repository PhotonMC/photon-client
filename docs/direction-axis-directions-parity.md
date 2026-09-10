# P01.c — Direction axis-directions Rust davranış kanıtı

`src/direction.rs` içindeki `Axis.get_positive`, `get_negative` ve `get_directions`, Java eksen/yön eşlemelerini ve pozitif/negatif dizi sırasını uygular. Rust adapter `direction_axis_directions` JSONL operation'ı ile yön snapshot'larını dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar X, Y ve Z eksenlerinin tekil pozitif/negatif yönlerini ve birleşik dizi sırasını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `4f7b352046e51ad873047d8c1b31b450095876f829ccbaf3482743512da2c481`, birleşik capture kimliği `38ffe865bdd9dd0cb6ae9a43c2001e20f39cee76a5ec14f4911b3bfecbeaa43e` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-directions eşlemelerini kapsar; predicate, plane, random ve coordinate seçimleri ayrı davranış sözleşmeleridir.
