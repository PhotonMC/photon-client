# P01.c — Direction nearest Rust davranış kanıtı

`src/direction.rs` içindeki `get_nearest`, Java `Direction.getNearest` üç bileşenli overload'unun strict eksen karşılaştırmasını, işaret yönünü ve nullable fallback davranışını uygular. Rust adapter `direction_nearest` JSONL operation'ı ile hem bileşen hem Vec3i forwarding senaryolarını karşılaştırır; Java oracle iki overload'u doğrudan çağırır.

Senaryolar baskın eksenleri, eşit büyüklükleri, null fallback'i ve i32 minimum değerindeki Java mutlak değer davranışını kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `1dfb7a76f46a9ea8ea2868e2cbbe435f134086ea822738929c5e6533081458fa`, birleşik capture kimliği `e8b519abb2579853c8aca622c3159adbd5f756f45e6ea05504029c364a49a25f` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getNearest` overload'larının yön sonucunu kapsar; approximate-nearest ve entity bakış akışları ayrı davranış sözleşmeleridir.
