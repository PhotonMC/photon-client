# P01.d — AABB segment clipping Rust davranış dilimi

`src/aabb.rs`, Java `AABB.clip(Vec3, Vec3)` yolunun eksen sırasını, eşiklerini, epsilon'lı yüz kontrollerini ve ilk geçerli segment ölçeğini uygular. `examples/behavior_adapter.rs` sonucu f64 bitleriyle JSONL senaryolarına bağlar; beklenen değer üretmez.

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
- Altı ailenin birleşik corpus'u **836/836** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `b04328d3836db8458e37a31e45b96c1c92ba29348f35c055b49ac522f339a43e`.

Birleşik referans kimliği: `cadb5899f013b31a0f9a6ac389c7c724dd7eb83f79803c37c0b757d7e852454e`.

Yerel raporlar: `compare-71jdlsjf` (math debug), `compare-owaw1c7e` (math release), `compare-2pxxyovy` (birleşik debug) ve `compare-peecwwsu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt 6 yeni tek kutu clip senaryosuyla sınırlıdır. Çoklu shape, `BlockHitResult`, yüz yönü ve istemci hedefleme davranışları sonraki dilimlerdir.
