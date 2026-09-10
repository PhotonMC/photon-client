# P01.c — BlockPos vector yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_vector_helpers` işlemi, BlockPos'ın Vec3i offset/subtract forwarding'ini ve scalar multiply yolunu Java oracle ile karşılaştırır.

Java math capture kimliği `1cf201798a30a0d8cc2392ea50cb2c7c4aa0b5c595d1d2953964c9591129c7d2` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `869668c9ff9a2a713bee0bacb11c3fbfba9740ef43b6dbe378c6b705ad460d81` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos vector delta ve scalar multiply yardımcılarını kapsar; Vec3i'nin bağımsız yordamları ayrı dilimlerdedir.
