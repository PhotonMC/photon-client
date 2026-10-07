# P09.af — Entity dimensions refresh ve client reapply — [client-parity] birleşik kayıt

Bu kayıt, Java `Entity.refreshDimensions` içindeki yeni dimensions/eye-height
yazımını, aynı konumda bounding box yenilemesini ve server-side resize düzeltme
gate'ini Rust world yardımcısıyla karşılaştırır. Kanıt sınıfı `client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:3377-3405`, eski dimensions'ı saklar, pose'tan yeni
  dimensions'ı alır, dimensions ve eye height state'ini yazar, ardından
  `reapplyPosition()` çağırır. Sonraki gate yalnız server level, ilk tick değil,
  physics açık, küçük ölçü, büyüme ve Player olmayan entity koşullarını birlikte
  ister.
- `world/entity/Entity.java:485-488`, `reapplyPosition()` aynı konumla `setPos`
  çağırarak yeni dimensions'a göre bounding box kurar ve son bilinen konumu
  temizler.
- `world/entity/EntityDimensions.java:6-50`, dimensions constructor'larını ve
  feet-origin `makeBoundingBox` yolunu tanımlar.

## Davranış sözleşmesi

- Refresh sonunda dimensions ve eye height yeni ölçüden gelir; bounding box
  mevcut entity konumunda yeni ölçüyle yeniden kurulur.
- `reapplyPosition` konumu değiştirmez. Bu kayıtta gözlenen
  `position_after_refresh`, giriş konumuyla bit düzeyinde aynıdır.
- `should_fudge_position`, şu Java koşullarının birleşimidir: level client-side
  değil, first tick false, no-physics false, yeni width ve height en fazla `4.0F`,
  yeni ölçü eski ölçüden width veya height olarak büyük ve entity Player değil.
- Client level'da büyüyen ölçü için bile fudge gate false kalır. Server-side
  gate'in true olduğu durum, sonraki gerçek `fudgePositionAfterSizeChange` dünya
  sorgusunun policy girdisidir; bu kayıt onun konum düzeltmesini yürütmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`shape/entity-dimensions-refresh/client-reapply` kaydı yedi özgün gözlem taşır:

| Gözlem | Client/server | Java `should_fudge_position` |
| --- | --- | :---: |
| client-growth-reapplies-box | client, ölçü büyüyor | false |
| client-shrink-to-sleeping | client, ölçü küçülüyor | false |
| server-growth-eligible | server, küçük non-Player ölçü büyüyor | true |
| server-player-excluded | server, Player | false |
| server-first-tick-gate | server, ilk tick | false |
| server-no-physics-gate | server, no-physics | false |
| server-large-dimensions-gate | server, width `4.1F` | false |

Her satır eski/yeni dimensions, aynı konum ve yeni bounding box snapshot'ını
taşır. Dimensions ve box uçları golden çıktıda raw f32/f64 olarak saklanır.

Java oracle, gerçek `EntityDimensions` constructor, `withEyeHeight` ve
`makeBoundingBox` metotlarını çağırır; `refreshDimensions` içindeki state yazımı,
`reapplyPosition` sonucu ve fudge gate'i kaynak sırasıyla uygular. Beklenen
sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`a21af76318605c83fb1ec49d2327c13b647138129044449f1af6039c2a1a8456`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2947 kayıt
karşılaştırıldı.

## Rust doğrulaması

`refresh_entity_dimensions`, yeni dimensions ile feet-origin bounding box'ı
aynı konumda kurar ve Java'nın server-side fudge koşullarını tek production
helper içinde değerlendirir. Client adapter, eski/yeni ölçüleri, konumu, box'ı
ve gate sonucunu aynı JSON sözleşmesiyle karşılaştırır.

Debug adapter SHA-256 değeri
`eccdc2a1e6228355ef3994d0911ec3830f357a8f20b8fee4bd885b6b28ca3dde` ile
**2947/2947** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`50e7cd511138274f69323debbf43b1b45075b749a6b18626e140842ffa3bc267` ile
aynı capture üzerinde **2947/2947** eşleşti; fark `null` kaldı.

Çalıştırılan dilim kontrolleri:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
cargo test --locked --offline refresh_entity_dimensions_reapplies_box_and_client_gate
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Bu kayıt gerçek `ClientEntity` dimensions/eye-height/bounding-box alanlarına
packet ve pose metadata'sı üzerinden bağlanmayı, server-side
`fudgePositionAfterSizeChange` free-position aramasını, supporting state'i,
attachment ölçeklenmesini ve tam entity tick akışını kapsamaz. Bunlar P09'un
açık world-state bağımlılıklarıdır; P10 yerel hareketi için collision-aware
oyuncu tick/input bağlantısı ayrıca gereklidir.
