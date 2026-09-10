# P01.c — Direction axis-orientation Rust davranış kanıtı

`src/direction.rs` içindeki `Axis.is_vertical` ve `is_horizontal`, Java enum predicate'lerinin üç eksendeki sonuçlarını uygular. Rust adapter `direction_axis_orientation` JSONL operation'ı ile iki boolean sonucu dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar X, Y ve Z eksenlerinin dikey/yatay sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `74b808f59d73d90fc04e7c4eb3f5989ed2ecb60babf1889877bb88d5e7766bc3`, birleşik capture kimliği `021b9773c00ad5713ff63f4ea7be38710c58e8afccc9b129d2974803f6fa7877` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca axis-orientation predicate'lerini kapsar; eksen yönü, plane, random ve coordinate seçimleri ayrı davranış sözleşmeleridir.
