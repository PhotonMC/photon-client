# P05.c — KeyboardInput sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java istemcisinin `KeyboardInput.tick()` tuş snapshot’ını ve iki eksenli hareket vektörünü kapsar. Gerçek pencere, GLFW olayları, tuş tekrarları ve odak geçişleri P05.b/d kapsamındadır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/player/KeyboardInput.java:7-36`, `client/player/ClientInput.java:5-27` ve `world/entity/player/Input.java:5-13` incelendi. Oracle, kontrollü `Options` içindeki `KeyMapping` nesneleri üzerinden gerçek `KeyboardInput.tick()` çağrısını yapar.

- `tick()` yedi değeri sırasıyla `forward`, `backward`, `left`, `right`, `jump`, `shift`, `sprint` alanlarına taşır.
- Aynı eksenin iki yönü birlikte basılıysa impulse `0.0F` olur; yalnızca pozitif veya negatif yön basılıysa impulse sırasıyla `1.0F` veya `-1.0F` olur.
- Hareket vektörü `(leftImpulse, forwardImpulse)` bileşikleriyle kurulur ve Java `Vec2.normalized()` yolundan geçer. Tek yön uzunluğu `1`, iki dik yön birlikte `1/sqrt(2)` sonucunu verir; sıfır vektör sıfır kalır.
- Jump, shift ve sprint hareket vektörünü değiştirmez; yalnızca `Input` snapshot’ında taşınır.

## JSONL sınırı

İstek `family: "math"`, `operation: "keyboard_input"` ve yedi boolean tuş alanını taşır: `up`, `down`, `left`, `right`, `jump`, `shift`, `sprint`. Sonuç `key_presses` içinde Java `Input` alanlarını, `move_vector` içinde f32 bitleriyle Vec2 `x`, `y`, `length` ve `length_squared` değerlerini taşır.

Rust karşılığı `crates/voxel-world/src/input.rs` içindeki `Input` ve `KeyboardInput` tipleridir. `KeyboardInput::new` kontrollü tuş durumunu alır; `tick` snapshot ve hareket vektörünü üretir. Adapter, sonucu Java oracle şemasıyla karşılaştırır.


## Client-parity doğrulaması

`crates/voxel-world/src/input.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre `KeyboardInput.tick()` snapshot’ını ve normalize edilmiş hareket vektörünü uygular. Java oracle, kontrollü `KeyMapping` durumlarıyla gerçek `KeyboardInput.tick()` çağrısını çalıştırır; beklenen sonuç Rust adapter’ından türetilmez.

Java referansı `b8ac51bd8c3bd440346514a4021569847347a2d7fedd0409b4fc84e14c8cc9f6` capture’ında iki tekrarla kararlı kaldı. Debug adapter karşılaştırması `compare-058fr0qu`, release adapter karşılaştırması `compare-_6wiu7ho` ve tam Java doğrulaması `verify-j9ido3zp` raporlarında eşleşti.

Doğrulanan senaryolar karşıt yönlerin sıfırlanmasını, tek ve çapraz hareketleri, dört hareket yönünü birlikte, jump/shift/sprint alanlarını ve hareket bayraklarıyla birlikte snapshot üretimini kapsar. Gerçek pencere olayları, GLFW odak geçişleri ve tuş tekrarları P05.b/d kapsamındadır; oynanabilir istemci bağlantısı henüz yoktur.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
