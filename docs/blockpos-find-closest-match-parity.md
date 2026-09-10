# P01.c — BlockPos findClosestMatch Rust davranış kanıtı

Rust adapter `blockpos_find_closest_match` işlemi, eşitlik predicate'iyle ilk Manhattan eşleşmesini Java oracle ile karşılaştırır.

Java math capture kimliği `eb48af1e939a1134e04e3c62c429ae66161f024855377cd609092b944ac8707d` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug/release math adapter karşılaştırmaları fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt ilk eşleşme, boş Optional, sıfır yarıçap ve i32 sınır yollarını kapsar; diğer predicate callback'leri ayrı kapsamdır.

Birleşik capture kimliği `bf0c111eb2762fc08b3f09a2eefcb6036a98ca411a5ef33bfa0d1a106ad8b020` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
