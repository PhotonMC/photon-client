# P01.c — Direction step Rust davranış kanıtı

Rust adapter `direction_step` işlemi, Direction’ın i32 adım getter'larını ve f32 step vektörünü Java `getStepX/Y/Z` ve `step()` çağrılarıyla karşılaştırır.

Java math capture kimliği `0223f82bc89c86357ba4b5147e4713b6b2e463f1872919bb21cd7076306c66dc` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `c5794094082a1506652a7f250204d50d0871bae63d80945997938db764059d33` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction step getter ve vektör yüzlerini kapsar; unit vector getter'ları ve diğer Direction davranışları ayrı dilimlerdedir.
