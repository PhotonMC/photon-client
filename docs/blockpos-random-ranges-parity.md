# P01.c — BlockPos random range Rust davranış kanıtı

Rust adapter `blockpos_random_ranges` işlemi, `randomBetweenClosed` ve `randomInCube` sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `56b3daf65646309d91103555670e86d200d948a91693d48937031ba1dd38eac9` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt finite limit ve geçerli bounded aralıklarla random draw sırasını, limit kısa yolunu ve `randomInCube` merkez forwarding'ini kapsar; invalid bound exception ayrıntıları kapsam dışıdır.

Birleşik capture kimliği `3051db48b90ed8689e2a77545c64b335b50076ec13f7757a51a94a0b03539131` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
