# P01.d — AABB move sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.move(double,double,double)` overload'unun her endpoint'e eksen bazında delta eklemesini ve sonucu AABB kurucusundan geçirmesini içerir. `BlockPos`, `Vec3` ve `Vector3f` overload'ları için ayrı [sözleşme](aabb-move-overloads-conformance.md) vardır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:223-225` incelendi. `MathOracle` doğrudan scalar `AABB.move` overload'unu çağırır; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Her min ve max endpoint'ine aynı eksendeki delta eklenir.
- Sonuç Java `AABB` kurucusundan geçer; sonlu delta ile endpoint sırası korunur.
- İşlem IEEE-754 double toplama sırasını korur. Signed zero ve büyük delta'larda ortaya çıkan bitler gözlenebilir çıktının parçasıdır.
- Zincir adımları bir önceki taşıma sonucunu sonraki `move` çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Yön | 3 | Pozitif, negatif ve karışık eksen delta'ları |
| Sayısal ayrıntı | 2 | Signed zero ve büyük double delta'sı |
| Zincir | 1 | Ardışık taşıma çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "move"` ve f64 bit string `x`, `y`, `z` delta'ları bulunur. Her adım sonrası kutu uçları `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Move senaryolarını içeren math referans kimliği `34b14a75d8c53b3350feba4a21f2e7648537e482ea15a67806d62235270bebad` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 394/394, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.move(double,double,double)` metodunun her endpoint'e eksen bazında delta eklenmesini uygular. `examples/behavior_adapter.rs` `move` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 394 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **394/394** math senaryosunda eşleşti.
- Altı yeni senaryoda pozitif/negatif/karışık delta, signed zero, büyük double delta'sı ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `34b14a75d8c53b3350feba4a21f2e7648537e482ea15a67806d62235270bebad`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-rgmctv_7` (math debug), `compare-w4quj7cd` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.move(double,double,double)` ve mevcut AABB adım adaptörüyle sınırlıdır. `BlockPos`, `Vec3` ve `Vector3f` overload'ları için [ayrı kanıt](aabb-move-overloads-conformance.md) vardır; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
