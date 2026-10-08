# P09.ad — Entity zero-Y collision state retention — [client-parity] birleşik kayıt

Bu kayıt, Java `Entity.move` içindeki çözülen hareketten collision state üreten
ve sıfır dikey hareket için önceki dikey state'i koruyan dalı Rust world
yardımcısıyla karşılaştırır. Kanıt sınıfı `client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:712-768`, `move` çözülen hareketten X/Z, Y ve
  yatay collision alanlarını üretir; `Math.abs(delta.y) > 0.0` ile yetkili
  instance kapısı dikey state'in ne zaman yenileneceğini belirler.
- `world/entity/Entity.java:672-680`, yenileme dalında `onGround` ve yatay
  collision state'ini yazar; ayrıca supporting-block yan etkisini başlatır.
- `world/entity/Entity.java:1245-1260`, oracle'ın kullandığı eksen sıralı
  `collideWithShapes` primitive'ini taşır.
- `util/Mth.java:158-164`, X/Z için `Mth.equal(double, double)` eşiğini
  `abs(b - a) < 1.0E-5` olarak tanımlar.

## Davranış sözleşmesi

- X ve Z collision alanları her çağrıda istenen delta ile çözülen hareketin
  `Mth.equal` karşılaştırmasından yeniden üretilir.
- Y collision alanı, istenen ve çözülen Y bileşenlerinin doğrudan eşitsizliğidir;
  `horizontalCollision`, X veya Z sonucunun OR birleşimidir.
- `verticalCollision`, `verticalCollisionBelow` ve `onGround` alanları önceki
  state'ten başlatılır.
- `Math.abs(delta.y) > 0.0 || isLocalInstanceAuthoritative()` doğruysa bu üç
  alan yeniden hesaplanır: dikey collision Y eşitsizliği, aşağı collision
  bunun negatif delta ile birleşimi, `onGround` ise aynı aşağı collision
  sonucudur.
- Sıfır Y'li remote hareket, önceki dikey state'i korur; aynı hareket yetkili
  instance'ta yeniden hesaplanır. Non-zero Y, remote instance'ta da yeniden
  hesaplama dalına girer.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`shape/entity-movement-state/zero-y-retention` kaydı dört özgün gözlem taşır:

| Gözlem | Java sonucu |
| --- | --- |
| zero-y-remote-retains-ground | X çarpışması true; önceki `verticalCollision`, `verticalCollisionBelow` ve `onGround` true kalır |
| zero-y-authoritative-recomputes | X çarpışması true; yeniden hesaplanan dikey alanlar false olur |
| nonzero-y-remote-recomputes-falling | `(0, -1, 0)`; dikey ve aşağı collision ile `onGround` true |
| nonzero-y-remote-recomputes-rising | `(0, 1, 0)`; dikey collision true, aşağı collision ve `onGround` false |

Java oracle, pinned istemci sınıfındaki gerçek `Entity.collideWithShapes`
private metodunu reflection ile çağırır; state retention korumasını
`Entity.move` kaynak sırasındaki guard ile uygular. Beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`e5e13f2c644f2dbec045ccd639356d6b0884bdaef05e9ad38b922d4982d793e6`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2945 kayıt
karşılaştırıldı.

## Rust doğrulaması

`apply_entity_movement_state`, Java'nın X/Z toleransını, doğrudan Y
karşılaştırmasını ve zero-Y authority guard'ını tek production helper içinde
uygular. Supporting-block state'i henüz ClientWorld'e bağlanmadığı için bu
kayıt yalnızca gözlenebilir collision alanlarını karşılaştırır.

Debug adapter SHA-256 değeri
`3bd9b261d43cfd89c0ace114c82258250a50d1dad568819298902adff4a9cf1d` ile
**2945/2945** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`b2698d05405f422e572d847e2bcf4767eb7bb8af249bec18d17f9171e8bebd56` ile
aynı capture üzerinde **2945/2945** eşleşti; fark `null` kaldı.

Çalıştırılan dilim kontrolleri:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /tmp/photon-jdk-wrapper
cargo test --locked --offline zero_y_remote_movement_retains_vertical_state
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/release/examples/behavior_adapter
```

Depo kontrolleri de geçti:

```text
cargo fmt --all -- --check
cargo test --locked --offline
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --examples
python3 tools/behavior/test_runner.py
python3 tools/behavior/run.py verify --family math --jdk-home /tmp/photon-jdk-wrapper
git diff --check
```

## Açık kapsam

Bu kayıt gerçek `Entity.move` içindeki level/block/entity/world-border
collider toplama çağrılarını, `setOnGroundWithMovement` supporting-block yan
etkisini, fall damage ve movement emission dallarını, `LocalPlayer` bağlantısını
ve tam tick akışını kapsamaz. Bunlar P09.a–P09.ac kayıtlarında kısmen ölçülen
veya roadmap'te açık kalan bağımlılıklardır; P10 yerel hareketi için gerçek
oyuncu tick/input bağlantısı ayrıca gereklidir.
