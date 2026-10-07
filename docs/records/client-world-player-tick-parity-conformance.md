# P08.e — [client-parity] ClientWorld local-player tick state

Bu kayıt, aynı sentetik tick izinin Java oracle'ı ve Rust `ClientWorld` kopyası
üzerinde yürütülmesini karşılaştırır. Kapsam, local player için tick sayacı,
previous transform, üç adımlı head rotation interpolation ve istemci effect
sürelerinin sorgulanabilir state'e yazılmasıdır.

Bu kayıt tam `LocalPlayer.tick` veya oynanabilir istemci eşitliği iddiası
taşımaz. Hareket, collision, paket gönderimi ve görsel sunum ayrı açık
kapsamlardır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 / protocol 776 / Java 25 kaynaklarında şu dallar
izlendi:

- `client/multiplayer/ClientLevel.java:467-477`, tick edilen entity için
  `setOldPosAndRot`, `tickCount` artışı ve entity tick çağrısı sırasını tanımlar.
- `world/entity/Entity.java:1816-1825`, position ve body rotation previous
  state'inin tick başında kopyalanmasını tanımlar.
- `world/entity/LivingEntity.java:409-499`, client effect tick'ini ve head
  previous state güncellemesini tanımlar.
- `world/entity/LivingEntity.java:3053-3056,3312-3315,3994-3995`, üç adımlı
  head hedefinin wrapped `rotLerp` ile ilerlemesini tanımlar.
- `world/effect/MobEffectInstance.java:220-243`, `tickClient` duration ve
  infinite duration davranışını tanımlar.

Oracle fixture, tam client runtime'ındaki renderer/profiler bağımlılıklarını
çalıştırmadan bu gerçek Java metotlarını çağırır: `setOldPosAndRot`,
`lerpHeadTo`, `lerpHeadRotationStep` ve `MobEffectInstance.tickClient`.
Fixture tick'i, `ClientLevel.tickNonPassenger` sırasındaki üç outer state
adımını kontrollü entity üzerinde uygular; movement ve ağ yan etkileri bu
kaydın kapsamına alınmaz.

## Davranış sözleşmesi

- Her local-player tick'inde `tick_count` bir artar.
- Tick başında current position ve yaw/pitch previous state'e kopyalanır;
  tick yapılmadan gelen position/rotation güncellemesi current state'te
  kalır ve bir sonraki tick'te previous state olur.
- Head hedefi `lerpHeadTo` ile üç adım olarak kurulduğunda her tick'te kalan
  adım sayısına göre Java wrapped `rotLerp` uygulanır. Previous head state,
  tick başındaki current head değeridir.
- Sonlu effect duration her client tick'inde bir azalır; `-1` infinite
  duration korunur. Süresi `0` olan kayıt bu dar state modelinde tutulur.
- Aynı effect ID yeniden uygulandığında state, aynı listedeki kaydın yerine
  geçer; liste sırası korunur.

## Yeni dal ve varyant kararı

İlk kez Java `ClientLevel` tick sırası ile local player state geçmişi aynı
sentetik trace'te karşılaştırıldı. Önceki `client-world-tick-conformance.md`
kaydı yalnız vanilla sunucuya bağlı `server-conformance` kanıtıdır; önceki
`client-parity` kayıtları bu tick state sonucunu taşımadığı için aynı sonucu
veren kayıt sayısı **0**'dır.

Tek parametrik `client-world/player-tick-state` kaydı dört gözlem adımı taşır:
head hedefi ve effect başlangıcı, tick öncesi transform güncellemesi, hedefin
tamamlanması ve effect replacement. Sabit veya aynı sonucu üreten yeni bir
entity/effect varyantı açılmadı.

## Java yakalama

Senaryo girdisi `tools/behavior/cases.py` içinde özgün ve sentetiktir. Başlangıç
state'i `(10,64,-2)`, yaw/pitch `(350,-20)`, head `350` ve biri finite, biri
infinite iki effect içerir. Dört operasyon toplamında position/rotation
değişimi, head hedefi `135`, iki tick'lik tamamlanma ve effect ID `18` için
replacement gözlenir.

**Tarih:** 2026-09-15

**Java referans kimliği:**
`08fb45506de4c057358bf099732f01442c3d96936d48ea83afad584cf2daed50`

Java oracle iki tekrar çalıştırıldı. Math corpus'u **2978** kayıt olarak
kararlı kaldı ve tekrar farkı `null` oldu.

## Rust doğrulaması

Adapter, üretim `ClientWorld::tick` yolunu ve local-player world state
uygulama yollarını kullanır. `client-world/player-tick-state` çıktısı
position, previous position, yaw/pitch, previous yaw/pitch, head state ve
effect state alanlarında Java capture ile karşılaştırılır.

Debug adapter SHA-256:
`405640dbee5f47bf8ea31a6825104b57b43ab9a422071cc814c67ef520bc7b6f`

```text
python3 tools/behavior/run.py compare --family math \\
  --adapter target/debug/examples/behavior_adapter
case_count=2978
status=parity_passed
difference=null
```

Release adapter SHA-256:
`683dc76b5b1d2b1d5af1216c41e6bcb31369326364f4d6ccdccf5d22e0dc942e`

Release karşılaştırması aynı capture'ta **2978/2978** kayıt ve `difference=null`
ile `parity_passed` oldu. Java `verify` adımı da iki tekrar sonunda aynı
referans kimliğini ve `difference=null` sonucunu korudu.

Son kontroller:

- `cargo test --locked --offline --quiet`: **193 geçti**.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: geçti.
- `cargo fmt --all -- --check`: geçti.
- `git diff --check`: geçti.
- `python3 tools/behavior/test_runner.py`: **5 geçti**.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
Rust adapter'a verilir. Vanilla sunucusuna canlı bağlantı bu sonucun parçası
değildir; canlı vanilla koşuları wire kabulü için ayrı `server-conformance`
kanıtı olarak tutulur.

## Açık kapsam

Gerçek `LocalPlayer.aiStep` input snapshot/timing, sprint trigger window,
travel ve collision, fluid/block lookup, movement packet eşikleri, effect
particle lifecycle, passenger tick, full entity lifecycle ve görsel client
henüz bu kayıtla kapanmaz.
