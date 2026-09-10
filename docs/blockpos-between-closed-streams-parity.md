# P01.c — BlockPos betweenClosedStream Rust davranış kanıtı

Rust adapter `blockpos_between_closed_streams` işlemi dört stream overload'unu Java oracle ile karşılaştırır.

Java math capture kimliği `d30d65d9b4bd9942adff6eb2baa1aefa6a01d87168a03ee36ed8855dfe389bad` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt BlockPos, BoundingBox, AABB ve int coordinate stream overload'larının finite küçük girdilerdeki forwarding'ini kapsar.

Birleşik capture kimliği `2bbcd4fb142817a005f14a5179e10209375a77701cd9346de056d1a974b1fcdb` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
