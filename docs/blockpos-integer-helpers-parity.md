# P01.c — BlockPos integer yardımcıları Rust davranış kanıtı

Rust adapter `blockpos_integer_helpers` işlemi, BlockPos/Vec3i Manhattan ve chessboard metriklerini ve Axis bileşen seçimini Java oracle ile karşılaştırır.

Java math capture kimliği `8feb0d77cfb5cf566a8bbb15ef28089effec5413f6f3d984309cdd53df5ba085` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `036f9d2f57d05c5d132f1a57c3c25bad8d5030cf644cd0fa2d5a152f7be4f8f7` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos integer yardımcılarını kapsar; Vec3i'nin f64 mesafe yolları ve kalan yardımcıları ayrı dilimlerdedir.
