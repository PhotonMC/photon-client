# P13 — Vanilla survival inventory minimum attack charge component correct prediction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her geçerli oturumda player bağlandıktan sonra vanilla server console'undan şu
hazırlık komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone[minimum_attack_charge=0.5]
```

Java oracle, `MINIMUM_ATTACK_CHARGE=0.5` component hash'ini `0x9ec7d769` verdi.
Geçerli iki temiz probe koşusu:

```text
cargo run --offline --release --example survival_inventory_minimum_attack_charge_component_correct_prediction_probe -- 127.0.0.1:25566 InvMinChargeA
cargo run --offline --release --example survival_inventory_minimum_attack_charge_component_correct_prediction_probe -- 127.0.0.1:25566 InvMinChargeB
```

`InvAttackCharge2` ve `InvAttackCharge3` adları daha önceki test state'lerinden
kalan hazırlık slot güncellemeleri taşıdığı için kanıta alınmadı. Bu iki
exploratory koşuda hedef type `7` öncesinde sırasıyla type `4` ve type `1,2`
gözlendi; sabit state `2` ile gönderilen click full resync üretti. Temiz adlar
ile hazırlık tek state `2` slot gözlemi verdi.

## Sonuçlar

| Gözlem | InvMinChargeA | InvMinChargeB |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added7 hash=0x9ec7d769,removed0` | `item1,count1,added7 hash=0x9ec7d769,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive7` | `state=2, slot36=count1,item1,positive7` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2689` | `2209` |

Her iki geçerli vanilla oturumunda doğru state-id ve doğru float component
hash'iyle click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya
cursor düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı
`0` kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `MINIMUM_ATTACK_CHARGE` component
hash'i carried `HashedStack` içinde doğru eşleştiğinde slot/cursor correction
paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival inventory
minimum attack charge component correct prediction contract belgesindedir](survival-inventory-minimum-attack-charge-component-correct-prediction-contract.md).

Diğer float değerleri, diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları, değişen slot haritasının varyantları ve
özel menu akışları açık kalır.
