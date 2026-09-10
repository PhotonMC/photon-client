# P01.c — BlockPos betweenClosed AABB Rust davranış kanıtı

Rust adapter `blockpos_between_closed_aabb` işlemi, AABB uçlarının floor dönüşümünü ve inclusive BlockPos traversal'ını Java oracle ile karşılaştırır.

Java math capture kimliği `e7910890bfd37b6cf2c46e9fda186fb61be379d695c909b1235d1ba79bf1c025` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu küçük AABB'leri kapsar; BoundingBox/stream ve büyük veya sonsuz aralıklar ayrı kapsamdır.

Birleşik capture kimliği `b88bab35dcec6d29f33c93274632b3bb9a433620ae69e00d2498abc5a02ee835` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
