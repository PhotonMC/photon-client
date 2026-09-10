# P01.c — BlockPos geometry yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_geometry_helpers` işlemi, BlockPos rotate/cross/atY yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `42a6ce4635c7072299aaed2523c40a5b4590f224094c628af3435e3f6a7c26f4` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `190e7958671c8092dc326700c20b06e16c0ae714617e2b138073197d1c3da510` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos geometry yardımcılarını kapsar; Rotation enum davranışının diğer yolları ayrı dilimlerdedir.
