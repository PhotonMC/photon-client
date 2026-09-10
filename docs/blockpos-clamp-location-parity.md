# P01.c — BlockPos clamp-location Rust davranış kanıtı

Rust adapter `blockpos_clamp_location` işlemi, BlockPos'ın Vec3 clamp sınırlarını Java oracle ile karşılaştırır.

Java math capture kimliği `57dbb7055dab2c862b74ea0af09fd3a7c0e6f47b86f7e60e87fedf3eb31d1b89` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `60f33aa8068c7d1544ac42265a52afa6d4dc54da3856c21feb7b1eab237d1be2` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos clamp-location yardımcı yolunu kapsar; diğer Mth clamp overload'ları ayrı dilimlerdedir.
