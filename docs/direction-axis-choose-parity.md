# P01.c — Direction axis-choose Rust davranış kanıtı

`src/direction.rs` içindeki `Axis.choose_i32`, `choose_f64` ve `choose_bool`, Java `Direction.Axis.choose` overload'larının x/y/z seçim davranışını uygular. Rust adapter `direction_axis_choose` JSONL operation'ı ile üç sonucu dışarı verir; Java oracle overload'ları doğrudan çağırır.

Senaryolar üç eksendeki ayırt edici değerleri, signed i32 sınırlarını ve özel f64 bitlerini Java oracle çıktısıyla karşılaştırır.

Java math capture kimliği `f5118835b6978b7f3daf585aaa981852456e02841b516150b3a8fba3685b1747`, birleşik capture kimliği `7afa4dfb0877ea063c2fafe11985de74d797d985562f727d6bb15b286c892367` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Axis.choose overload'larını kapsar; predicate, yön eşlemeleri, plane ve diğer Direction sorguları ayrı davranış sözleşmeleridir.
