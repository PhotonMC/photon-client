# P01.c — BlockPos coordinate helpers Rust davranış kanıtı

Rust adapter `blockpos_coordinate_helpers` işlemi, containing ve componentwise min/max yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `cf1f618be0f377747e5a836bbc7e26ef4aaa766fa6e8a860a8051d98862bbff6` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu double floor, Vec3 Position forwarding ve componentwise integer min/max yollarını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `f49fd734c2cfc969856e3c0a90e314629b2e517e32cbd1ed1b8f5f55658853fc`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
