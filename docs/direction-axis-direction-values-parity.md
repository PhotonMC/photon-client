# P01.c — Direction axis-direction-values Rust davranış kanıtı

`src/direction.rs` içindeki `AxisDirection.get_step` ve `opposite`, Java enum step değerlerini, adlarını ve karşıtlık ilişkisini uygular. Rust adapter `direction_axis_direction_values` JSONL operation'ı ile bu alanları dışarı verir; Java oracle aynı enum yöntemlerini doğrudan çağırır.

Senaryolar POSITIVE ve NEGATIVE değerlerinin step, ad ve karşıt sonuçlarını Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `3e211af4291f51617cbb425f3c78bb3f4269726871ff3187b972beab0700d53f`, birleşik capture kimliği `ce65510770b37fb2ffa769c1f83b93c1ca651ebc4699a41192b45ba78541fbe7` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca AxisDirection değer yollarını kapsar; eksen yön eşlemeleri, predicate, plane ve coordinate seçimleri ayrı davranış sözleşmeleridir.
