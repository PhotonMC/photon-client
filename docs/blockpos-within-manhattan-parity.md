# P01.c — BlockPos withinManhattan Rust davranış kanıtı

Rust adapter `blockpos_within_manhattan` işlemi, iterable ve sequential stream sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `f2171426d9af3c802c99c5b0378ff74fb602d45bbccf5d72aec8b5d5a6bf0a69` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu, non-negative reach değerleriyle traversal sırasını, z mirror çiftlerini ve origin offset sarmasını kapsar; callback/predicate ve taşmalı dev reach'ler ayrı kapsamdır.

Birleşik capture kimliği `ddbd12001da620eec4b0db4a4ffd76e8f2dce41e0148d13db1129db91231db09` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
