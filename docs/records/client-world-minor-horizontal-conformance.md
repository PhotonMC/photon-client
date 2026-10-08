# P09.ab — LocalPlayer minor-horizontal collision — [client-parity] birleşik kayıt

Bu kayıt, Java `LocalPlayer` sınıfının yatay çarpışmayı “minor” sayma kararını
Rust world hareket yardımcılarıyla karşılaştırır. Karar, çarpışma sonrası gerçek
yatay hareket ile yaw'a göre dönüştürülmüş input vektörünün açısına bağlıdır.
Kanıt sınıfı `client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `client/player/LocalPlayer.java:1104-1118`, `isHorizontalCollisionMinor`
  metodunun yaw dönüşümünü, iki yatay uzunluk kapısını, dot/acos hesabını ve
  strict eşik karşılaştırmasını belirler.
- `world/entity/Entity.java:760-773`, yatay collision oluştuğunda alt sınıf
  kararını çağırır; yatay collision yoksa minor alanını false yapar.
- `client/player/LocalPlayer.java:924`, bu alanı sprint bırakma kararında kullanır.
- `util/Mth.java:50-56`, sinüs ve kosinüsü quantized f32 lookup tablosundan
  döndürür.
- `client/player/LocalPlayer.java:115`, açı eşiğini `0.13962634F` olarak
  sabitler; metodun length kapıları `1.0E-5F` kullanır.

## Davranış sözleşmesi

- Yaw, Java'nın `(float)(Math.PI / 180.0)` çarpımıyla radyana çevrilir ve
  `Mth.sin/cos` lookup sonuçları f64 ara hesaplara yükseltilir.
- Input `(xxa, zza)` yaw ile dünya eksenine dönüştürülür. Dönüştürülmüş input
  veya gerçek hareketin yatay uzunluğu `1.0E-5F` değerinden küçükse sonuç
  false'tur.
- Aksi halde dot product, iki uzunluğun karekökü ve `acos` hesaplanır. Açı
  `0.13962634F` değerinden küçükse sonuç true'dur; eşitlikte sonuç false'tur.
- Bu kayıt, `Entity.move` içindeki base collision flag üretimini yeniden
  ölçmez; LocalPlayer alt sınıf override'ını ve onun boolean sonucunu ölçer.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/local-player-minor-horizontal` kaydı on gözlem taşır:

| Gözlem grubu | Java sonucu |
| --- | --- |
| aligned-forward, aligned-yaw-90, aligned-strafe-yaw-45 | `true` |
| seven-point-nine-degrees | `true` |
| eight-degree-threshold, eight-point-one-degrees | `false` |
| reverse-movement | `false` |
| zero-input, zero-movement, subthreshold-input | `false` |

Java oracle, pinned JAR içindeki gerçek `LocalPlayer.isHorizontalCollisionMinor`
metodunu reflection ile çağırır; yalnızca gerekli yaw, input ve movement alanları
kontrollü fixture üzerinde kurulur. Beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`1d86cd5b9f37ad650c19c4d647db63f819fb5f42dbd7eec0d0fe18a8ecdbb1aa`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2939 kayıt
karşılaştırıldı.

## Rust doğrulaması

`is_minor_horizontal_collision`, Java'nın Mth sin/cos lookup yardımcılarını,
f32 yaw/radian ara değerini, f64 vektör işlemlerini, strict length kapılarını
ve 8 derece eşiğini aynı sırayla uygular. Behavior adapter bu sonucu aynı
parametrik input tablosu üzerinden yayınlar.

Debug adapter SHA-256 değeri
`be85ae5c09bc5faadd96b22f590b6a3cd5b5b43cc81eadc736e973860016d8c7` ile
**2939/2939** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`27a335f29383c6913c760eb0de377c740c8781b2ab4fae2348fd497ad95cf920` ile aynı
capture üzerinde **2939/2939** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /tmp/photon-jdk-wrapper
cargo test --locked --offline local_player_minor_horizontal_collision_follows_java_angle_gate
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Bu kayıt, `Entity.move` içindeki step-up sonrası hareketin LocalPlayer'a
bağlanmasını, sprint state transition'ını, gerçek input snapshot/tick akışını,
collision shape toplama yollarını ve diğer entity alt sınıflarını kapsamaz.
P09.g base collision flags kaydıyla birlikte okunmalıdır.
