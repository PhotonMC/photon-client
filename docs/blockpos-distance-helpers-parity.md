# P01.c — BlockPos distance yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_distance_helpers` işlemi, BlockPos/Vec3i düşük köşe ve merkez mesafelerini ve strict `closerThan` eşiğini Java oracle ile karşılaştırır.

Java math capture kimliği `0cbd9c4149fa6d8b3e90ae52b0656899f46a0646c8bdb0eec6351f3296588c95` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `06a9a25116ae4456c9ba8ad15c9397d96d3a7a95fee964411bdd267b46491d3d` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos distance yardımcılarını kapsar; Vec3i'nin diğer mesafe ve codec yolları ayrı dilimlerdedir.
