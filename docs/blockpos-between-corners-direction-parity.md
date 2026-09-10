# P01.c — BlockPos between corners in direction Rust davranış kanıtı

Rust adapter `blockpos_between_corners_direction` işlemi, AABB, BlockPos ve ham koordinat overload'larını Java oracle ile karşılaştırır.

Java math capture kimliği `12fd4fac2ab5b4c789f5326d449ac86f9727e7ab6c66cb23a0cda6291d545d03` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt AABB floor forwarding'ini, normalize edilmiş köşeleri, Y-X-Z/Y-Z-X eksen önceliğini, yön işaretlerini, signed zero/NaN dallarını ve i32 fark taşmasını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `a484457fbd28df4964559a6bd7ceb4bf1a54df8b623de1cb84048c96b7df3f38`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
