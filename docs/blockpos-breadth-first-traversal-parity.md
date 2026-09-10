# P01.c — BlockPos breadth-first traversal Rust davranış kanıtı

Rust adapter `blockpos_breadth_first_traversal` işlemi, Java'nın FIFO kuyruk ve packed visited traversal'ını sentetik grafiklerle karşılaştırır.

Java math capture kimliği `3727446d35576418209105f5c151ee1e3c8b99eb95e0ef13e2b16e640c2195c1` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt processor/provider sırasını, FIFO komşu ekleme düzenini, SKIP/STOP dallarını, depth/count sınırlarını ve packed long visited anahtarını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `48c013300094ef4882ade1cee5d8ac9569b681c9ca3f31ddcc1fa275e3a1e44d`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
