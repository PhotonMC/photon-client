# P01.a — `Mth.getSeed(int, int, int)` Rust davranış kanıtı

`src/math.rs` içindeki `mth_get_seed`, Java `Mth.getSeed(int, int, int)` koordinat overload'unun i32 ve i64 taşma sırasını uygular. `src/random.rs` positional factory'si de aynı yardımcıyı çağırır; ayrı bir seed formülü tutulmaz.

Java oracle `Mth.getSeed(x, y, z)` çağrısını doğrudan yürütür. Girdilerde sıfır, işaretli i32 sınırları, `x * 3129871` i32 taşması ve büyük karışık koordinatlar bulunur. Beklenen sonuçlar Java oracle çıktısından alındı.

Java referans capture kimliği:

`19ecd56114bfeca6651a19ed8170f87981da3d7b7e9998f3aa0e880ac3a311a8`

Yeni birleşik capture kimliği:

`d29deef1874c152ce4860c236af9875602b0f92e24aa57b9ded67ac5d4c379cb`

Math adaptörüyle karşılaştırma geçti:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Birleşik capture, debug ve release adaptörleriyle de karşılaştırıldı:

```bash
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
```

Bu kanıt koordinat overload'u ile sınırlıdır. `getSeed(Vec3i)` forwarding'i ve world/positional random tüketim sırası daha geniş veri akışlarının parçasıdır.
