# P09.ae — LivingEntity dimensions scale composition — [client-parity] birleşik kayıt

Bu kayıt, Java `LivingEntity.getDimensions` içindeki sleeping özel yolunu ve
normal pozda age/entity scale bileşimini Rust client-world dimensions
yardımcısıyla karşılaştırır. Kanıt sınıfı `client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/LivingEntity.java:3737-3742`, sleeping pozunda sabit sleeping
  dimensions'ı döndürür; diğer pozlarda varsayılan dimensions'ı entity scale'i
  ile çarpar. Varsayılan dimensions yolu age scale'i içerir.
- `world/entity/LivingEntity.java:554-565`, baby için `getAgeScale()` değerini
  `0.5F`, yetişkin için `1.0F` seçer ve attribute değerinden gelen entity
  scale'ini döndürür.
- `world/entity/EntityDimensions.java:6-50`, f32 width/height/eye-height
  çarpım sırasını, fixed ölçünün scale erken dönüşünü ve feet-origin box'ı
  tanımlar.
- `world/entity/Avatar.java:18-37,65-68`, oyuncu pozlarının varsayılan
  dimensions tablosunu sağlar; bu kayıt tablodan gelen ölçüyü bağımsız fixture
  olarak besler.

## Davranış sözleşmesi

- `Pose.SLEEPING` seçildiğinde `LivingEntity.SLEEPING_DIMENSIONS` doğrudan
  kullanılır: width, height ve eye height `0.2F`, fixed true; age ve entity
  scale'leri bu dalda okunmaz.
- Diğer pozlarda önce `defaultDimensions.scale(ageScale)`, sonra elde edilen
  sonucu `.scale(entityScale)` çağrısı izler. Her iki çağrı da aynı f32
  constructor/scale semantiğini kullanır.
- Default dimensions fixed ise age ve entity scale çağrıları ölçüyü değiştirmez.
  Scalable ölçüde width, height ve eye height sırasıyla ölçeklenir.
- Bounding box feet-origin'dir; yeni ölçünün width yarısı ve height değeri
  entity konumuna Java AABB uçları olarak taşınır.
- Bu kayıt `getAgeScale` baby kararını, attribute map effective value ve
  `sanitizeScale` override'ını değil, bu yolların çözdüğü scale değerlerinin
  `getDimensions` içindeki birleşimini ölçer.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`shape/living-entity-dimensions/scale-branches` kaydı dört özgün gözlem taşır:

| Gözlem | Pose | Dimensions sonucu (raw f32: width, height, eye height; fixed) |
| --- | ---: | --- |
| regular-age-then-entity-scale | 0 | `3ee66667`, `3faccccc`, `3f9b851f`; false |
| sleeping-ignores-both-scales | 2 | `3e4ccccd`, `3e4ccccd`, `3e4ccccd`; true |
| fixed-default-ignores-entity-scale | 5 | `3f800000`, `40000000`, `3fd9999a`; true |
| regular-height-only-rounding | 3 | `3f2eb851`, `3f919999`, `3f51a9fb`; false |

Her satır ayrıca farklı negatif/ondalıklı konumda feet-origin bounding box
snapshot'ı taşır. Box uçları ve pose ID çözümü golden çıktıda raw f64 olarak
saklanır.

Java oracle, gerçek `EntityDimensions` constructor, `withEyeHeight` ve `scale`
metotlarını çağırır; `LivingEntity.getDimensions` içindeki sleeping/normal
branch'i kaynak sırasıyla uygular. Beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`2fbbc51e1db02cc9e274cd0ef583080c3351196cebffcc4885939aad363e6bb3`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2946 kayıt
karşılaştırıldı.

## Rust doğrulaması

`living_entity_dimensions`, sleeping için Java'nın sabit dimensions yolunu;
diğer pozlar için age scale ardından entity scale çağrılarını production
yardımcısında uygular. `ClientEntityDimensions` raw f32 alanları ve
`make_bounding_box` çıktısı aynı adapter üzerinden karşılaştırılır.

Debug adapter SHA-256 değeri
`2b8bc3b690de8e68e54154b4e48c904a8c286c217831bc556911f191ff061679` ile
**2946/2946** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`6d9a3ea5b5d47fe3f2df7ac8b308a4429f475936b542b502adbbbdb88bd1e459` ile
aynı capture üzerinde **2946/2946** eşleşti; fark `null` kaldı.

Çalıştırılan dilim kontrolleri:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /tmp/photon-jdk-wrapper
cargo test --locked --offline living_entity_dimensions_follow_sleep_and_scale_branches
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

Bu kayıt attribute map effective scale hesabını, `sanitizeScale` tür override'larını,
full EntityType/default dimensions registry'sini, dinamik baby mob ölçülerini,
`refreshDimensions` ile eski/yeni box ve konum yan etkilerini, attachment
ölçeklenmesini ve gerçek entity tick bağlantısını kapsamaz. Bunlar P09'un açık
world-state bağımlılıklarıdır; P10 yerel hareketi için collision-aware oyuncu
tick/input bağlantısı ayrıca gereklidir.
