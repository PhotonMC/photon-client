# P01.c — Direction facing-angle Rust davranış kanıtı

`src/direction.rs` içindeki `is_facing_angle`, Java `Direction.isFacingAngle(float)` derece/radyan f32 dönüşümünü, Mth sinüs/kosinüs lookup yolunu, yön normalinin X/Z ürünlerini ve strict pozitiflik kararını uygular. Rust adapter `direction_is_facing_angle` JSONL operation'ı ile boolean sonucu dışarı verir; Java oracle aynı Direction yöntemini doğrudan çağırır.

Senaryolar yatay ve dikey yönleri, cardinal/diagonal/negatif açıları, signed zero, subnormal, sonsuzluk, NaN ve f32 sınırlarını kapsar. Beklenen boolean sonuçlar Java oracle çıktısından alınır.

Java math capture kimliği `10123a3164046b82429b3881a351d93b17b5e7d74bb0381b4376a9580c3c16ab`, birleşik capture kimliği `c3e71daf3bde4157f96bdcd8e7e99da4aa758649aa8ee7ac499bb7de6aa348e0` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca facing-angle sonucunu kapsar; diğer Direction bulma, dönüştürme ve entity bakış akışları ayrı davranış sözleşmeleridir.
