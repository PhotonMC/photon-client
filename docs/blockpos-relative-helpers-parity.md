# P01.c — BlockPos relative yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_relative_helpers` işlemi, BlockPos Direction/Axis relative overload'larını ve altı cardinal yönün tek/sayılı yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `28d59732d2db48e98cee214e142821c2f33b77011d49e204e3a91a407d7d8617` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `ea457925c7c014e392cfb2c4499d58a3e1a9777f6a25c83b14f0f045d028dc96` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos relative ve cardinal yardımcılarını kapsar; diğer BlockPos dönüşümleri ayrı dilimlerdedir.
