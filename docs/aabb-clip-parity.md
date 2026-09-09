# P01.d — AABB segment clipping Rust davranış dilimi

`src/aabb.rs`, Java `AABB.clip(Vec3, Vec3)` yolunun eksen sırasını, eşiklerini, epsilon'lı yüz kontrollerini ve ilk geçerli segment ölçeğini uygular. Koordinat endpoint'li statik overload [ayrı dilimde](aabb-clip-coordinates-parity.md) uygulanır. `examples/behavior_adapter.rs` sonucu f64 bitleriyle JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `AABB.java` içindeki public clip ve private eksen kesme yardımcıları incelendi. Math oracle doğrudan sabitlenen Java JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 6 senaryoluk kapsam [referans belgesindedir](aabb-clip-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `Aabb::clip` | `Vec3` doğru parçasının kutuya ilk geçerli kesişim noktasını veya yok sonucunu döndürür |
| `clip_point` | `1e-7` delta eşiği, `0 < s < scale` koşulu ve diğer eksenlerde `1e-7` toleransı |

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 323 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **323/323** eşleşti: önceki 317 math senaryosu ve 6 yeni AABB clip senaryosu.
- Altı ailenin birleşik corpus'u **1000/1000** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `b04328d3820db8458e37a31e45b96c1c92ba29348f35c055b49ac522f339a43e`.

Birleşik referans kimliği: `b624d0ee4bd5155be0aa8e190a91cc25d84867d5c6f605db7c91ad8c599196f2`.

Yerel raporlar: `compare-71jdlsjf` (math debug), `compare-owaw1c7e` (math release), `compare-ag226u45` (birleşik debug) ve `compare-8116vkuu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt 6 yeni tek kutu `AABB.clip(Vec3, Vec3)` senaryosuyla sınırlıdır. Koordinat endpoint'li overload [ayrı kanıttadır](aabb-clip-coordinates-parity.md); VoxelShape clip tamamlandı; occlusion ve istemci hedefleme davranışları sonraki dilimlerdir.
