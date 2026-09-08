# P01.d — AABB move sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-move-parity.md)). Kapsam, Java `AABB.move(double,double,double)` overload'unun her endpoint'e eksen bazında delta eklemesini ve sonucu AABB kurucusundan geçirmesini içerir. `BlockPos`, `Vec3` ve `Vector3f` overload'ları için ayrı [sözleşme](aabb-move-overloads-contract.md) vardır.

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

Move senaryolarını içeren math referans kimliği `34b14a75d8c53b3350feba4a21f2e7648537e482ea15a67806d62235270bebad` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `4ce00821c27ab9db6c47c791dd4c5cde73722cdbac10509ba375a4a2bd135af7` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 394/394, birleşik corpus'ta 902/902 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
