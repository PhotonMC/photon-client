# P01.c — Direction axial-rotation Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::clockwise` ve `counter_clockwise`, Java eksen özel dönüş tablolarını ve hizalı yön kısa yolunu uygular. Rust adapter `direction_axial_rotation` JSONL operation'ı ile iki sonucu dışarı verir; Java oracle iki enum metodunu doğrudan çağırır.

On sekiz yön/eksen çifti Java oracle çıktısıyla karşılaştırılır.

Java math capture kimliği `b281ec0375183bbbdf0f5b15a1313103c7bd368f271b87a176a54c3b2c7d3786`, birleşik capture kimliği `41d6c2ceb63f7b055b5ab39658bf532208416282dd760be52177ee330023666b` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca eksenli clockwise/counter-clockwise yolları kapsar; eksensiz dikey hata yolları ve matrix rotation ayrı davranış sözleşmeleridir.
