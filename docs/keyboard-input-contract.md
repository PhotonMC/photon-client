# P05.c — KeyboardInput sözleşmesi

Bu dilim, Java istemcisinin `KeyboardInput.tick()` tuş snapshot’ını ve iki eksenli hareket vektörünü kapsar. Gerçek pencere, GLFW olayları, tuş tekrarları ve odak geçişleri P05.b/d kapsamındadır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/player/KeyboardInput.java:7-36`, `client/player/ClientInput.java:5-27` ve `world/entity/player/Input.java:5-13` incelendi. Oracle, kontrollü `Options` içindeki `KeyMapping` nesneleri üzerinden gerçek `KeyboardInput.tick()` çağrısını yapar.

- `tick()` yedi değeri sırasıyla `forward`, `backward`, `left`, `right`, `jump`, `shift`, `sprint` alanlarına taşır.
- Aynı eksenin iki yönü birlikte basılıysa impulse `0.0F` olur; yalnızca pozitif veya negatif yön basılıysa impulse sırasıyla `1.0F` veya `-1.0F` olur.
- Hareket vektörü `(leftImpulse, forwardImpulse)` bileşikleriyle kurulur ve Java `Vec2.normalized()` yolundan geçer. Tek yön uzunluğu `1`, iki dik yön birlikte `1/sqrt(2)` sonucunu verir; sıfır vektör sıfır kalır.
- Jump, shift ve sprint hareket vektörünü değiştirmez; yalnızca `Input` snapshot’ında taşınır.

## JSONL sınırı

İstek `family: "math"`, `operation: "keyboard_input"` ve yedi boolean tuş alanını taşır: `up`, `down`, `left`, `right`, `jump`, `shift`, `sprint`. Sonuç `key_presses` içinde Java `Input` alanlarını, `move_vector` içinde f32 bitleriyle Vec2 `x`, `y`, `length` ve `length_squared` değerlerini taşır.

Rust karşılığı `src/input.rs` içindeki `Input` ve `KeyboardInput` tipleridir. `KeyboardInput::new` kontrollü tuş durumunu alır; `tick` snapshot ve hareket vektörünü üretir. Adapter, sonucu Java oracle şemasıyla karşılaştırır.
