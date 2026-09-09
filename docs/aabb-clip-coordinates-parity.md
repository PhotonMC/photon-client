# P01.d — AABB koordinat clip overloadu doğrulaması

`src/aabb.rs`, Java `AABB.clip(double, double, double, double, double, double, Vec3, Vec3)` statik yolunu ham endpoint'ler, eksen eşikleri, epsilon'lı yüz kontrolleri ve en yakın geçerli segment ölçeğiyle uygular. `examples/behavior_adapter.rs` bu yolu JSONL math adımı olarak çağırır.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `AABB.java:298-311` içindeki koordinat overloadu ile `getDirection` ve `clipPoint` yardımcıları incelendi. Math oracle doğrudan sabitlenen Java JAR sınıfını çağırır. Ayrıntılı sözleşme ve 6 senaryoluk kapsam [referans belgesindedir](aabb-clip-coordinates-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `Aabb::clip_coords(min, max, from, to)` | Ham min/max koordinatlarını normalize etmeden doğru parçasının ilk geçerli kesişim noktasını veya yok sonucunu döndürür |
| `clip_point` | `1e-7` delta eşiği, `0 < s < scale` koşulu ve diğer eksenlerde `1e-7` toleransı |

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 468 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **468/468** math senaryosunda eşleşti.
- Altı yeni senaryoda iki yönlü geçiş, diagonal tarama, paralel miss, ham ters endpoint'ler, eşik altı delta, sıfır hareket, NaN ve sonsuzluk karşılaştırıldı.
- Altı ailenin birleşik corpus'u **1064/1064** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `30579b0337c9cbef64bfe390fc1b1e3c6fb20fd2e3c1d1f1ebb1626d7805d642`.

Birleşik referans kimliği: `298fc79fff700683b37658552439ad616697eca9abe5ae8a7cd43d2e28ce910e`.

Yerel raporlar: `compare-z0upb0ml` (math debug), `compare-0by5xb21` (math release), `compare-45pvo1ui` (birleşik debug) ve `compare-bodmv5fk` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca statik koordinat clip overloadu ve mevcut AABB adım adaptörüyle sınırlıdır. VoxelShape clip tamamlandı; occlusion ve istemci hedefleme davranışları sonraki dilimlerdir.
