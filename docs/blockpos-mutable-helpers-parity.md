# P01.c — BlockPos mutable helpers Rust davranış kanıtı

Rust adapter `blockpos_mutable_helpers` işlemi, MutableBlockPos constructor, mutation ve immutable-result yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `f0efc0bd24b33e5bc88940e628a54342c8f3e4d440f1341bc44a419b54d0d2e2` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt floor/packed set yollarını, offset/move/clamp state değişimini, i32 sarmasını ve mutable override'larının immutable dönüşlerini kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `55adde7cba770b1ef6a8ccd4f1c273c13a5a4294399ac117cbdbd31596c530f9`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
