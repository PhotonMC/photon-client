# P09.f — Oyuncu desired pose seçimi — [client-parity] birleşik kayıt

Bu kayıt, Java `Player.updatePlayerPose` içindeki desired pose önceliğini,
swimming fit kapısını ve crouching/swimming fallback kararını Rust'a taşır.
Kanıt sınıfı `client-parity`'dir: aynı sentetik durum tabloları Java 26.2
oracle'ında ve Rust adapter'ında çalıştırılıp karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/player/Player.java:343-355`, önce swimming ölçüsünün
  sığmasını kontrol eder; swimming kutusu sığmıyorsa pose değiştirmez.
  Sığıyorsa spectator/passenger veya desired pose için uygunlukla birlikte
  desired, crouching ve swimming fallback sırasını seçer.
- `world/entity/player/Player.java:359-370`, desired pose önceliğini sleeping,
  swimming, fall-flying, spin-attack, crouching ve standing olarak belirler.
  Crouching, shift basılı ve abilities flying kapalı olduğunda seçilir.
- `world/entity/player/Player.java:284`, bu güncellemenin Player tick'inin
  sonunda çağrıldığını gösterir.
- `world/entity/Entity.java:2666-2670,2707-2723` ve
  `world/entity/LivingEntity.java:3662-3668`, passenger, shift, swimming ve
  fall-flying durumlarının kaynak state'lerini tanımlar.

## Sözleşme

- Desired pose sırası sleeping > swimming > fall-flying > spin-attack >
  crouching > standing'dir. Shift basılı olsa bile abilities flying açıksa
  crouching seçilmez.
- `swimming_fits` false ise güncelleme sonucu yoktur; mevcut pose korunacağı
  için helper `None` döndürür.
- Swimming ölçüsü sığıyorsa spectator veya passenger desired pose uygunluk
  sonucunu bypass eder. Diğer durumda desired ölçüsü sığarsa desired seçilir.
- Desired ölçüsü sığmaz ve crouching ölçüsü sığarsa crouching seçilir; ikisi
  de sığmazsa swimming seçilir.
- Rust `ClientPlayerPoseInput::desired_pose` ve `updated_pose` bu dalları tek
  veri yapısında taşır. Gerçek local-player flags, `ClientWorld` collider
  sorgusu ve pose state'ine yazma ayrı bağlantı adımlarıdır.

## Test girdileri ve Java yakalama

Tek parametrik özgün girdi `tools/behavior/cases.py` içindeki
`shape/player-pose-selection/priority-fit-fallbacks` kaydındadır. On iki tablo
satırı tüm desired önceliklerini, flying istisnasını, swimming fit kapısını,
iki fallback'i ve spectator/passenger bypass dallarını aynı karşılaştırmada
ölçer. Her satır `desired_id` ve güncelleme uygulanmışsa `actual_id` döndürür.

| Girdi | Desired | Güncellenen pose |
| --- | ---: | ---: |
| `sleeping-priority` | 2 | 2 |
| `swimming-priority` | 3 | 3 |
| `fall-flying-priority` | 1 | 1 |
| `spin-attack-priority` | 4 | 4 |
| `crouching` | 5 | 5 |
| `flying-clears-crouch` | 0 | 0 |
| `standing-default` | 0 | 0 |
| `swimming-fit-gate` | 0 | yok |
| `fallback-to-crouching` | 0 | 5 |
| `fallback-to-swimming` | 0 | 3 |
| `spectator-bypass` | 0 | 0 |
| `passenger-bypass` | 0 | 0 |

Java referans kimliği:
`928b1b6a3c8c1702e5ba0b8e14280bead1154ac73b0a4c2fd960819cc4861fc3`.
Capture, Java'nın aynı girdilerle iki ardışık çalıştırmasında kararlı kaldı.

## Rust doğrulaması

`ClientPlayerPoseInput` Java branch sırasını ve fit fallback'lerini uygular.
Adapter aynı on iki girdiyi production helper'a verir; Java oracle ile sonuçlar
**2915/2915** eşleşti. Son doğrulamadaki adapter SHA-256 değeri
`66da33a66ab0a86ca925dcbc784b4df6662025f4bce226ce9e535b6b64fa56d4`.

```
cargo fmt --all -- --check
cargo test --locked --offline --all-targets --quiet
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --examples
python3 tools/behavior/run.py verify --family math
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Son Rust unit test toplamı 154 geçti.

## Açık kapsam

Local player flags ve abilities state'inin gerçek input/tick akışından
üretilmesi, `swimming_fits`/desired/crouching sonuçlarının block, entity ve
world-border collider'larına bağlanması, `setPose`/`refreshDimensions` eski-yeni
box yan etkileri, su içindeki swimming state'i, movement ve packet üretimi P09
ile P10/P11'in açık dallarıdır. Bu kayıt canlı sunucu kabulü değil, Java
istemcisiyle aynı sentetik girdinin karşılaştırılmasıdır.
