# P01.c — Direction get-y-rot Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::get_y_rot`, Java `Direction.getYRot(Direction)` yatay sabitlerini ve dikey hata yolunu uygular. Rust adapter `direction_get_y_rot` JSONL operation'ı üzerinden başarıda raw f32 bitlerini, hatada Java mesajını dışarı verir; Java oracle static enum metodunu doğrudan çağırır.

Altı yön adı test edildi; beklenen başarı ve hata sonuçları Java oracle çıktısından alındı.

Java math capture kimliği `42462ab197598295116b848be00dc9cb225cc5825dfc9b81fb9db149f7aa9fde`, birleşik capture kimliği `786e7ede06ae88d1ca9909909c689f4a36314a667ae7e86c99d700160cb96022` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getYRot` yolunu kapsar; diğer Direction açı ve entity bakış yolları ayrı davranış sözleşmeleridir.
