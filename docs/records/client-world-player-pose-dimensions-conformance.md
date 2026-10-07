# P09.e — Oyuncu pose ölçüleri — [client-parity] birleşik kayıt

Bu kayıt, Java istemcisindeki `Avatar` pose tablosunun oyuncu ölçülerini ve
bilinmeyen pose ID'lerinin çözümünü Rust `ClientWorld` durumuna taşır. Kanıt
sınıfı `client-parity`'dir: aynı pose ID dizisi Java 26.2 oracle'ında ve Rust
adapter'ında çalıştırılıp karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Avatar.java:18-37,65-68`, oyuncuya özgü pose ölçülerini
  kurar. Standing, crouching, fall-flying, swimming, spin-attack, sleeping ve
  dying dalları ayrı ölçü ve göz yüksekliği kullanır; tabloda bulunmayan pose
  standing ölçüsüne düşer.
- `world/entity/LivingEntity.java:191,3737-3743`, sleeping için özel ölçü
  yolunu, diğer pose'larda varsayılan ölçünün entity scale'iyle çarpılmasını
  belirler. Bu kayıtta oyuncunun varsayılan scale'i birdir.
- `world/entity/Pose.java:11-37`, pose ID aralığını ve ID'den enum çözümünü
  sağlar; aralık dışı ID standing'e çözülür.
- `world/entity/Entity.java:3377-3380`, pose metadata'sı değişince ölçü ve
  bounding box yenilemesinin tetiklendiğini gösterir. Yenileme yan etkileri bu
  dar kaydın dışında tutulur.

## Sözleşme

- Rust pose çözümü Java `Pose.BY_ID` davranışını izler; 0–17 dışındaki ID'ler
  standing olarak çözülür.
- Oyuncu ölçüleri feet-origin bounding box için şu tabloda sabittir:

  | Pose | ID | Width | Height | Eye height | Fixed |
  | --- | ---: | ---: | ---: | ---: | :---: |
  | Standing | 0 | 0.6 | 1.8 | 1.62 | hayır |
  | Fall flying | 1 | 0.6 | 0.6 | 0.4 | hayır |
  | Sleeping | 2 | 0.2 | 0.2 | 0.2 | evet |
  | Swimming | 3 | 0.6 | 0.6 | 0.4 | hayır |
  | Spin attack | 4 | 0.6 | 0.6 | 0.4 | hayır |
  | Crouching | 5 | 0.6 | 1.5 | 1.27 | hayır |
  | Long jumping, croaking, using tongue, sitting, roaring, sniffing, emerging, digging, sliding, shooting, inhaling | 6, 8–17 | 0.6 | 1.8 | 1.62 | standing fallback |
  | Dying | 7 | 0.2 | 0.2 | 1.62 | evet |

- `ClientPlayerState::player_dimensions`, mevcut local pose'tan bu tabloyu
  döndürür. Player type ID taşıyan remote entity için aynı sorgu
  `ClientEntity::player_dimensions` üzerinden yapılır; diğer entity türleri
  `None` döndürür.
- `make_bounding_box` ölçüleri feet-origin AABB'e taşır ve Java'nın f32 ölçü
  değerlerini f64 uçlara dönüştürme sırasını korur.
- Bu dilim desired pose seçimini, fit kontrolünü, entity/world-border
  collider'larını, scale/age dallarını ve `refreshDimensions` yan etkilerini
  kapatmaz.

## Test girdileri ve Java yakalama

Tek parametrik özgün girdi `tools/behavior/cases.py` içindeki
`shape/player-pose-dimensions/all-poses-and-fallback` kaydındadır. Aynı kayıt
18 geçerli ID'yi, `-1` ve `18` fallback ID'lerini, sabit bir negatif/ondalıklı
world position'ı ve her pose için dimensions ile bounding box snapshot'ını
karşılaştırır. Tek kayıt, aynı Java pose tablosunun sabitleri değişen kopyaları
yerine bütün ID dallarını kapsar.

| Girdi grubu | Java gözlemi |
| --- | --- |
| `0`, `5` | Standing ve crouching scalable ölçüleri |
| `1`, `3`, `4` | Aynı `.6 × .6` ölçü ve `.4` göz yüksekliği |
| `2`, `7` | Fixed ölçü; sleeping göz yüksekliği `.2`, dying göz yüksekliği `1.62` |
| `6`, `8–17` | Tabloda olmayan Avatar pose'ları standing'e düşer |
| `-1`, `18` | `Pose.BY_ID` sonucu ID `0`, standing |

Java referans kimliği:
`a8b70b6a7219d83487f42f91164249e17c413aab460e0ac4f050fa239317fdd7`.
Capture, Java'nın aynı girdilerle iki ardışık çalıştırmasında kararlı kaldı.

## Rust doğrulaması

`ClientEntityPose::player_dimensions`, Java Avatar tablosundaki scalable/fixed
ölçüleri ve göz yüksekliklerini raw f32 bitleriyle taşır. `ClientPlayerState` ve
oyuncu türündeki `ClientEntity`, bu sonucu sorgulanabilir world state'ine
bağlar. Adapter ile oracle aynı pose ID dizisini ve feet-origin box'larını
karşılaştırır. World modül testi açık pose dallarını, fallback ID'lerini ve
tablo dışında kalan pose'ları kontrol eder.

```text
cargo fmt --all -- --check
cargo test --locked --offline --all-targets --quiet
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --examples
python3 tools/behavior/run.py verify --family math
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math corpus'u 2914 kayıtla doğrulandı. Java ve Rust sonuçları **2914/2914**
eşleşti; son doğrulamadaki adapter SHA-256 değeri
`8220389bf305aa76eea9d04073aab80df5b633bd611bccd64806a63a76e962c0`.
Rust test toplamı 153 geçti.

## Açık kapsam

Player'ın desired pose seçimi ve `updatePlayerPose`, pose fit kararının block,
entity ve world-border collider'larıyla birlikte çalışması, scale/age/baby
ölçü dalları, `refreshDimensions` sonucu eski/yeni box state'i, gerçek entity
dimensions registry/context'i ve collision-aware hareket entegrasyonu P09'un
açık dallarıdır. Bu kayıt canlı sunucu kabulü değil, Java istemcisiyle aynı
sentetik girdinin karşılaştırılmasıdır.
