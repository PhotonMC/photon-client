# P01.c — Direction quaternion rotation Rust davranış kanıtı

`src/direction.rs` içindeki `Direction::get_rotation`, Java `Direction.getRotation()` için yön başına sabit JOML quaternion yolunu kullanır. JOML'nin yarım açı, `cosFromSin` işareti ve f32 saklama davranışı `src/math.rs` içindeki yardımcılarla taşınır. Adapter sonucu x/y/z/w raw f32 bitleri olarak dışarı verir; Java oracle aynı enum metodunu doğrudan çağırır.

Java math capture kimliği `e37b4611c76826a0b1e25c93a366285d488a856a012e4a9c65998d9e140f9697` olarak `.data/` altında tutulur. Capture iki ayrı JVM çalıştırmasında kararlı bulundu ve Rust debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `ddbcf0f3b1df30103517064c4eb9c6bfef9a057d1b10a4f86f786830300869cc` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Bu kanıt altı Direction quaternion sabitinin üretim sonucunu kapsar. Genel quaternion işlemleri, matrix dönüşümleri, Direction matrix rotation ve entity bakış yönü ayrı kapsamdır.
