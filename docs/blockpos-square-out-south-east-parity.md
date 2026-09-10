# P01.c — BlockPos squareOutSouthEast Rust davranış kanıtı

Rust adapter `blockpos_square_out_south_east` işlemi, deprecated dört elemanlı stream sırasını Java oracle ile karşılaştırır.

Java math capture kimliği `6fb8e81c6433b1449a63fad69e23d28ceb9d2dabcff2f18ee9ce87187983cb65` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca dört konumlu `squareOutSouthEast` sırasını kapsar; diğer BlockPos iterator'ları ayrı dilimlerdedir.

Birleşik capture kimliği `d198d18c2ddf3f2b80450d9f9b2ed9ee4218d0843ae84e02da47a67103b08bff` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
