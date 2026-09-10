# P01.c — BlockPos spiralAround Rust davranış kanıtı

Rust adapter `blockpos_spiral_around` işlemi, mutable spiral iterator snapshot'larını Java oracle ile karşılaştırır.

Java math capture kimliği `adf7d7acc4966c1a55b6f6b964b09659450d3875caf1e4b1084ead23f80428c1` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt geçerli, farklı eksenli yönlerle finite radius traversal'ını ve i32 koordinat sarmasını kapsar; aynı eksenli precondition hata metni kapsam dışıdır.

Birleşik capture kimliği `8641ac7b61b66bfcfac73fba4f0fe3f9b1a5ed23ca70994fb431c9ec203a5b1c` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
