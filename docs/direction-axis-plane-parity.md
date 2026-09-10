# P01.c — Direction axis-plane Rust davranış kanıtı

`src/direction.rs` içindeki `Axis.get_plane`, Java eksenlerini yatay/dikey Plane değerlerine eşler. Rust adapter `direction_axis_plane` JSONL operation'ı ile sonucu dışarı verir; Java oracle `Axis.getPlane()` yöntemini doğrudan çağırır.

Senaryolar X, Y ve Z eksenlerinin Plane sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `26a33290bf89d09112fec48aeb9ec0ebebc950f3264c5bbdd9203a3dfbdb85b2`, birleşik capture kimliği `d49ac1c6710e092d783f7e8f72d7c443377c2ef38c41a412f955fb1f0e3b7383` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Axis.getPlane eşlemesini kapsar; Plane'in diğer iterable, random, predicate ve stream yolları ayrı davranış sözleşmeleridir.
