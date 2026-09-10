# P01.c — AxisCycle Rust davranış kanıtı

Rust adapter `axis_cycle_helpers` işlemi, AxisCycle permütasyonlarını ve MutableBlockPos cycle setter'ını Java oracle ile karşılaştırır.

Java math capture kimliği `4e31a6e287300acbb9c7448ccd1d665e09941318f9d2eb5976824e659000cbd3` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt üç int/double permütasyonunu, axis yönlerini, inverse/between seçimlerini ve MutableBlockPos setter forwarding'ini kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `e524bd8c6548c77e9d64537affc2c803c904090c8a1e90891f5191acc901623e`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
