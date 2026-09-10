# P01.c — BlockPos packed yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_packed_helpers` işlemi, BlockPos packed sabitlerini, long alan erişimcilerini, pack/unpack yollarını, `getFlatIndex` maskesini ve iki packed offset overload'unu Java oracle ile karşılaştırır.

Java math capture kimliği `ac1376c29c59f59e206a5fa7eccd0a6924434b7d2439a83fb341819eb49227c5` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `9c9537578057644a61018d1c1717a79b2982142a19575719ef420eb00af836e8` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos packed yardımcılarını kapsar; diğer BlockPos geometri ve iterasyon yolları ayrı dilimlerdedir.
