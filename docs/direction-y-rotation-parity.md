# P01.c — Direction y-rotation Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::clockwise_y` ve `counter_clockwise_y`, Java eksensiz Y dönüş tablolarını ve dikey hata yollarını uygular. Rust adapter `direction_y_rotation` JSONL operation'ı ile iki sonucu dışarı verir; Java oracle iki enum metodunu doğrudan çağırır.

Altı yön girdisi, yatay dönüşler ve dikey hata mesajlarıyla Java oracle çıktısına karşılaştırılır.

Java math capture kimliği `ce6d3c2901a0c4eeb11a92ff58b6267db8d84bcdd4706068f6a906caab0e6d62`, birleşik capture kimliği `b9fe38e6188e7c1423ff30e78934b6daa161a516e44f8566db2d409d01148bae` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca eksensiz `getClockWise` ve `getCounterClockWise` yollarını kapsar; eksen parametreli dönüşler ayrı davranış sözleşmesidir.
