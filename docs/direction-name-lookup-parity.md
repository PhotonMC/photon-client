# P01.c — Direction ve Axis name lookup Rust davranış kanıtı

Rust adapter `direction_name_lookup` işlemi, `Direction::from_name` ve `Axis::from_name` yollarını Java `byName` lookup'larıyla karşılaştırır. Geçerli enumlarda ad, serialized name ve metin yüzleri; geçersiz adlarda null/`found: false` sonucu taşınır.

Java math capture kimliği `ec517b984123384c87f5509cd22e1828ce38ff75f205dd0f2ded4ab6f21d6da3` olarak `.data/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `490b72d70518ecaea73be8f84e7112792a18c7ca3ea95c5804a3dbf762ba6afb` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Bu kanıt yalnızca Direction ve Axis serialized-name lookup yüzlerini kapsar; codec DataResult dönüşümleri ve diğer enum davranışları ayrı kapsamdır.
