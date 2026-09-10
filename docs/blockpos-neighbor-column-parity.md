# P01.c — BlockPos neighbor-column Rust davranış kanıtı

Rust adapter `blockpos_neighbor_column` işlemi, beş yatay komşu grubunun inclusive Y traversal'ını Java oracle ile karşılaştırır.

Java math capture kimliği `e1be58bca503cae554c820096f196717da1af36c4d97f01a421eacd356df15f0` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `neighborColumn` finite listelerini, yatay grup sırasını ve artan/azalan Y traversal'ını kapsar; mutable API ve diğer BlockPos iterator'ları ayrı dilimlerdedir.

Birleşik capture kimliği `394ac801d0d22561c843054f24fad37fb75bc951c4c5f99f370250076c9b383d` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
