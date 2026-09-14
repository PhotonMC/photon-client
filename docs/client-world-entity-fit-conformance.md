# P09.d — Entity dimensions ve bloklara sığma — [client-parity] birleşik kayıt

Bu kayıt, Java istemcisinin `EntityDimensions` geometri primitive'ini ve player
pose sığma sorgusunun blok tarafını Rust client-world kopyasına taşır. Kanıt
sınıfı `client-parity`'dir: aynı sentetik ölçü, konum ve blok fixture'ı Java
26.2 oracle'ında ve Rust adapter'ında çalıştırılıp karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/EntityDimensions.java:6-22`, scalable/fixed dimensions için
  varsayılan göz yüksekliğini `height * 0.85F` ile kurar. Bounding box, entity
  konumunu ayak başlangıcı kabul eder; `width / 2.0F` ve `height` değerlerini
  AABB uçlarına taşır.
- `world/entity/EntityDimensions.java:25-50`, sabit olmayan ölçülerde genişlik,
  yükseklik ve göz yüksekliğini ayrı ölçekler; fixed ölçü `scale` çağrısını
  olduğu gibi döndürür; `withEyeHeight` yalnız göz yüksekliğini değiştirir.
- `world/entity/player/Player.java:343-375`, istenen pose için ölçülerden box
  üretip `deflate(1.0E-7)` uygular ve `CollisionGetter.noCollision` çağırır.
  `CollisionGetter.java:39-80` bu sorguyu blok, entity ve world-border
  kaynaklarına ayırır.

## Sözleşme

- Scalable/fixed constructor'ları width ve height'ı f32 olarak saklar; default
  eye height Java f32 çarpımıdır. `scale` fixed ölçüde erken döner, diğer
  ölçülerde width/height/eye height sonuçlarını Java çarpım sırasıyla üretir.
- `makeBoundingBox(position)` konumu feet origin olarak kullanır. Width'in f32
  yarısı ve height f32 değeri f64 AABB uçlarına taşınır; uçlar Java AABB
  constructor'ı gibi normalize edilir.
- Player fit sorgusu box'ın her eksenini `1.0E-7` içeri alır ve çakışma yoksa
  başarılıdır. Bu kayıtta fixture yalnız block collision üretir; entity ve
  world-border kaynakları sonraki world dilimlerinin sorumluluğundadır.
- Rust `ClientWorld::entity_fits_in_blocks`, aynı deflate sonrasında yüklenmiş
  block-state ID'lerini çağıranın shape tablosuyla sorgular. Eksik shape ve
  yüklenmemiş chunk Java fixture'ındaki air/no-collision davranışına karşılık
  gelir.

## Test girdileri ve Java yakalama

Altı özgün sentetik girdi `tools/behavior/cases.py` içindeki
`shape/entity-fit/*` kayıtlarında tutulur. İlk üç kayıt dimensions dallarını,
son üç kayıt block overlap, strict yüz teması ve epsilon içeri girişini ölçer.

| Senaryo | Dimensions çıktısı (width, height, eye height; raw f32) | Block fit |
| --- | --- | --- |
| `empty-default` | `3f19999a`, `3fe66666`, `3fc3d70a` | `true` |
| `scaled-and-eye-height` | `3e99999a`, `40666666`, `40300000` | `true` |
| `fixed-ignores-scale` | `3f800000`, `40000000`, `3fd9999a` | `true` |
| `overlap` | `3f19999a`, `3f800000`, `3f59999a` | `false` |
| `touching-face-after-deflate` | `3f800000`, `3f800000`, `3f59999a` | `true` |
| `epsilon-intrusion` | `3f800000`, `3f800000`, `3f59999a` | `false` |

Box ve deflate edilmiş query'nin raw f64 endpoint'leri golden çıktıda
saklanır; böylece f32 ölçülerin f64 AABB'e taşınması ve `1.0E-7` deflate
sonuçları da karşılaştırmaya dahildir.

Java referans kimliği:
`d5ad34965ca540a3903106f9b9e3b0225f6af02de90f6a2e5993e8cf82281486`.
Capture, Java'nın aynı girdilerle iki ardışık çalıştırmasında kararlı kaldı.

## Rust doğrulaması

`src/world.rs` içindeki `ClientEntityDimensions`, Java constructor, scale,
eye-height ve feet-origin box yollarını bit korumalı f32 alanlarıyla uygular.
`ClientWorld::entity_fits_in_blocks`, Player fit sorgusunun block-collision
alt yolunu üretir. World modül testi default eye height, f32→f64 box uçları,
fixed early return, strict temas ve epsilon içeri girişini kontrol eder.

```text
cargo fmt --all -- --check
cargo test --locked --offline --all-targets --quiet
cargo build --locked --offline --examples
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math corpus'u 2913 kayıtla doğrulandı. Java ve Rust sonuçları **2913/2913**
eşleşti; son doğrulamadaki adapter SHA-256 değeri
`e6e2259aab1149a51338f1c8f0ae4a9ccd4e85d4d2e20977b60cf59c2addc7c9`.

## Açık kapsam

Player'ın desired pose seçimi, gerçek entity type dimensions registry/context'i,
entity ve world-border collider'ları, block-state registry çözümü, on-ground ve
collision flag state'i, step-up adaylarının hareketle karşılaştırılması ve
collision-aware hareket entegrasyonu P09'un açık dallarıdır. Bu kayıt canlı
sunucu kabulü değil, Java istemcisiyle aynı sentetik girdinin karşılaştırılmasıdır.
