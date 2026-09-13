# P08.v — [server-conformance] LocalPlayer mob-effect duration tick'i

Bu dosya, local player üzerindeki mob-effect duration state'inin client tick
sınırında ilerletilmesini kaydeder. Canlı kanıt sınıfı `server-conformance`'dır:
Rust probe'u Mojang vanilla 26.2 dedicated server'ına bağlanır ve gelen effect
update/remove paketlerini `ClientWorld` state'iyle birlikte sorgular. Aynı
girdinin Java istemcisiyle karşılaştırılması yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`MobEffectInstance.java:175-205`, infinite duration'ın yalnızca `-1` olduğunu
ve duration sorgusunu tanımlar. `MobEffectInstance.java:240-270`, Java client
tick yolunda `tickClient()` çağrıldığında yalnızca remaining duration taşıyan
effect'in süresini bir azalttığını gösterir. `-1` değişmeden kalır; `0` veya
başka negatif değer remaining değildir. Client tick yolu duration `0` olan
effect kaydını map'ten silmez; ayrı remove packet'i gerekir.

`LivingEntity.java:480-505`, effect tick'inin living entity tick akışındaki
yerini; `LivingEntity.java:845-875`, client tarafında aktif effect listesinin
`tickClient()` ile tarandığını gösterir. `ClientLevel.java:450-477`, entity
tick sayacının client level tick sınırında artırıldığını ve ilgili entity tick
akışının burada yürüdüğünü gösterir.

## Sözleşme

- Her `ClientWorld::tick` çağrısında mevcut local player effect listesi bir kez
  client duration tick'inden geçirilir.
- Duration `> 0` ise bir azalır; `-1` infinite olarak aynı kalır.
- Duration `0` veya `-1` dışındaki negatif değerler azalmaz ve inactive kalır.
- Duration `0` olduğunda kayıt effect listesinden kaldırılmaz. Kaldırma,
  clientbound remove effect state'i uygulandığında gerçekleşir.
- Local player yoksa bu dilim effect duration state'i üretmez. Remote living
  entity effect ticking, hidden effect downgrade, blend state, parçacıklar,
  ikonlar ve gameplay etkileri bu dilimde uygulanmaz.

## Rust doğrulaması

`ClientMobEffectState::has_remaining_duration`, Java'nın infinite veya pozitif
duration koşulunu sorgulanabilir kılar. `tick_client`, yalnızca finite positive
duration'ı bir azaltır; expired effect kaydını silmez. `ClientWorld::tick`,
local player'ın tick sayacını ilerlettikten sonra mevcut effect kayıtlarını
bu yoldan geçirir. Remove packet'i daha sonra aynı state listesinden kaydı
kaldırabilir.

`local_player_mob_effect_durations_tick_without_removing_expired_entries`
unit testi finite `2 → 1 → 0`, infinite `-1`, zero `0` ve diğer negatif
duration'ı aynı koşuda doğrular; duration `0` kaydının üçüncü tick'te de listede
kaldığını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç oyun modu creative, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken vanilla konsolunda `P08vLive` için aşağıdaki hazırlık
yapıldı:

```text
gamemode survival P08vLive
item replace entity P08vLive hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]
effect give P08vLive minecraft:poison 60 0 true
```

Rust probe:

```text
cargo run --locked --offline --example survival_honey_bottle_poison_cleanup_probe -- 127.0.0.1:25568 P08vLive
```

Geçerli vanilla koşusunun önemli sonuçları:

| Gözlem | Sonuç |
| --- | ---: |
| login player ID | `2587` |
| mob-effect update | `entity=2587, effect=18, amplifier=0, duration=1200, flags=8, applied=true` |
| mob-effect remove | `entity=2587, effect=18, applied=true` |
| final local player effect count | `0` |
| honey bottle hazırlığı | `count=1, item=1412, components=[17]` |
| glass bottle remainder | `state=3, slot=36, count=1, item=1149, components=[]` |
| block changed ACK sequences | `[29]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` |
| received packet count | `7031` |

Bu canlı koşu packet update/remove akışının vanilla sunucudan alınıp local
player state'ine uygulanmasını ve remove sonrasında listenin boşalmasını
doğrular. Duration'ın client tick başına azalması canlı probe tarafından
doğrudan ölçülmedi; bu davranış Java kaynak incelemesine dayalı Rust unit
testinde doğrulandı. Dolayısıyla canlı kayıt wire/state conformance kanıtıdır,
Java istemcisiyle davranış eşitliği iddiası değildir.

## Açık kapsam

Gerçek `LivingEntity.tick` gövdesi, hidden effect zincirinin downgrade edilmesi,
blend state ve partikül yan etkileri, effect registry isim çözümlemesi, attribute
ve diğer gameplay etkileri, remote living entity effect state'i, render ikonları
ve `client-parity` karşılaştırması açık kalır.
