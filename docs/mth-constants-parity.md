# P01.a — Mth public constants doğrulaması

Java `Mth` scalar ve eksen sabitleri, Rust `MTH_*` API'siyle raw f32 bitleri üzerinden karşılaştırılır. Java oracle sabit alanlarını doğrudan okur; Rust adapter üretim sabitlerini JSONL'e taşır.

Java 25 / Linux x86_64 referansında scalar bitleri ve üç eksen vektörünün f32 bileşenleri debug ve release Rust adapter'larında eşleşti. Math capture kimliği `2e05c75c05118adf2dd2ca0cdc420fee4e30be9ff3ebb9c3ac0048aecbaf1af7`, birleşik capture kimliği `9c2116e5b40f80a6d1a615d8660ee3ffbc47cacb979b1a7bb998b9ba6cc696ce` olarak, kaynak ve girdilerle birlikte `.data/captures/behavior/` altında tutulur.

Doğrulama komutları:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca public sabit değerlerini kapsar; sabitlerin sonraki istemci ve matematik çağrı zincirlerindeki tüketicileri ayrı davranış dilimleridir.
