# P01.c — Direction alt enum metin Rust davranış kanıtı

Rust adapter `direction_enum_text` işlemi, `Direction.AxisDirection` ve `Direction.Plane` için Java enum adını ve `toString()` sonucunu karşılaştırır. AxisDirection için `getName()` yüzü de aynı sonuçta taşınır.

Java math capture kimliği `fd64bf62e981916ab8c6a92c9834c3ba532e49914347d37cb23c65a70090390d` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `6d816436db044dcfab0ec694e39d8f236761dd4ccc91f061688f8c6b59ecf83d` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca iki alt enumun ad ve metin yüzlerini kapsar; Direction'ın diğer davranışları ayrı davranış dilimlerindedir.
