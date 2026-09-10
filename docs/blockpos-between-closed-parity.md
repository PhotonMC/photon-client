# P01.c — BlockPos between-closed Rust davranış kanıtı

Rust adapter `blockpos_between_closed` işlemi, BlockPos inclusive range overload'larını Java oracle ile karşılaştırır.

Java math capture kimliği `0c2b4fba705ce69727eba2b2d4cd8a33879b1de13fcc17988d4e1c16eb08b518` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `00bd6c47a3e256fc3b8aa0f0cd020abced4e22cdb9ce64519f8b9e2e27e5aeb3` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca finite BlockPos between-closed listelerini ve x/y/z traversal sırasını kapsar; AABB/stream overload'ları ayrı dilimlerdedir.
