# P10.b — LocalPlayer applyInput — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LocalPlayer.applyInput` dalını P10.a'daki input
dönüşümünden sonra karşılaştırır ve sonucunu sorgulanabilir
`ClientPlayerState` alanlarına bağlar. Kanıt sınıfı `client-parity`'dir: aynı
sentetik `ClientInput` ve oyuncu state'i Java oracle'ında ve Rust production
yolunda çalıştırılır. Otomatik tuş örnekleme, tam LocalPlayer tick'i, fizik ve
serverbound hareket bu kaydın kapsamına girmez.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `client/player/LocalPlayer.java:692-705`, kameranın oyuncu tarafından
  kontrol edilip edilmediğine göre input uygulama yolunu seçer. Kontrollü dal
  P10.a dönüşümünü `xxa` ve `zza` alanlarına yazar, jump snapshot'ını
  `jumping` alanına taşır ve bob alanlarını rotasyona doğru yarım adımla
  günceller.
- `world/entity/LivingEntity.java:3179-3182`, kontrollü kamera dışındaki üst
  sınıf yolunda mevcut `xxa` ve `zza` değerlerini `0.98F` ile ölçekler; jump ve
  bob alanlarına dokunmaz.
- `client/player/ClientInput.java:5-27`, `moveVector` ile `keyPresses`
  snapshot'ının `applyInput` tarafından okunan iki kaynağını taşır.
- `world/phys/Vec2.java:59-70`, kontrollü dalın P10.a input dönüşümündeki
  normalize/karekök işlem sırasını sağlar. Bu dalın ayrı modifier davranışı
  [P10.a kaydında](client-player-input-conformance.md) tutulur.

## Davranış sözleşmesi

- Kamera kontrollü ise `moveVector`, P10.a dönüşümünden geçirilir ve sonucu
  mevcut `xxa/zza` değerlerinin üzerine yazılır.
- Aynı kontrollü dalda `jumping`, `keyPresses.jump()` snapshot'ından yazılır.
  `yBobO` ve `xBobO` önceki current bob değerlerini alır; sonra current bob
  bileşenleri kendi rotasyonlarına doğru `(rotation - bob) * 0.5F` adımıyla
  güncellenir.
- Kamera kontrollü değilse input vector ve jump snapshot'ı okunmaz. Üst sınıf
  yolu mevcut `xxa/zza` değerlerini kaynak sırasıyla `0.98F` ile çarpar,
  `jumping`, current bob ve previous bob değerlerini korur.
- Java f32 işlem sırası, signed zero ve ham özel değerler korunur; karşılaştırma
  epsilon kullanmadan f32 bitleriyle yapılır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik `player-input/apply` kaydı
beş gözlem taşır:

| Gözlem | İlk kez sınanan dal |
| --- | --- |
| `controlled-axis-updates-impulses-and-bob` | Kontrollü kamerada input, jump ve bob yazımı |
| `controlled-diagonal-sets-jumping` | Kontrollü diyagonal input ve jump snapshot'ı |
| `controlled-zero-preserves-signed-zero` | Kontrollü sıfır input'ta signed zero bob/impuls koruması |
| `uncontrolled-camera-uses-base-input` | `LivingEntity.applyInput` üst sınıf fallback'i |
| `controlled-overwrites-existing-impulses` | Kontrollü dalın eski impulsları ezmesi |

Java oracle, pinned JAR içindeki gerçek public `LocalPlayer.applyInput`
metodunu constructor çalıştırmadan ayrılmış fixture ve reflection ile çağırır.
Fixture `ClientInput` snapshot'ını, mevcut impulsları, jump/bob/rotasyon
alanlarını ve controlled-camera sonucunu açıkça kurar; beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`a2fed30b78fc4b25bd4d5369cc8e5091c8effaeaa407d6e826c924babe1842b8`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2951** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `apply_local_player_input`, P10.a helper'ını kontrollü
dalda yeniden kullanır ve üst sınıf fallback'ini ayrı uygular. `ClientWorld`
üzerindeki `apply_local_player_input` çağrısı sonucu
`ClientPlayerState` içindeki impuls, jump ve bob alanlarına yazar; state
getter'ları bu alanları sorgulanabilir kılar. Behavior adapter doğrudan aynı
production helper'ı çağırır.

Debug adapter SHA-256 değeri
`340ddd756e20e417259b8983d0c22ab3de0587e98952fc624eb437deb260328c` ile
**2951/2951** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`62f9747de557605b8480c90c63d17a441b9655d12b2b188eaa242acb7735c60d` ile
aynı capture üzerinde **2951/2951** eşleşti; fark `null` kaldı.

`world::tests::local_player_apply_input_updates_queryable_state_and_base_fallback`
testi, kontrollü ve üst sınıf yollarını gerçek `ClientWorld`/local player
state'inde raw f32 bitleriyle doğrular.

Dilimde kullanılan temel kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
cargo test --locked --offline local_player_apply_input_updates_queryable_state_and_base_fallback
```

## Vanilla server-conformance ayrımı

Bu dilim yeni bir wire veya serverbound movement davranışı bağlamaz; bu nedenle
P10.b sonucu canlı sunucu kabulü olarak raporlanmaz. Resmi Mojang vanilla 26.2
sunucusuna karşı yapılan movement/login adjunct'ı ve kapsam sınırı
[P10.a kaydında](client-player-input-conformance.md) tutulur. O kayıt
`server-conformance` wire akışını doğrular; bu belge `ClientPlayerState`
alanlarının Java istemcisiyle eşitliğini yalnızca yukarıdaki `client-parity`
capture'ı ile iddia eder.

## Açık kapsam

`applyInput` çağrısının gerçek keyboard snapshot'ına ve LocalPlayer tick'ine
bağlanması, sprint transition/aiStep, konum-hız entegrasyonu, collision ve
step-up, pose/crouch/jump çevre kapıları, su/lav/merdiven, uçuş/elytra,
araç kontrolü ve serverbound movement P10.c–e, P11 ve ilgili sonraki
dilimlerde açık kalır.
