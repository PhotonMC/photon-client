# P12 — Yaratıcı item stack ile blok yerleştirme sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, yaratıcı oyuncunun item stack'i hotbar'a koymasını, o hotbar slotunu seçmesini ve destek bloğunun yanına taş yerleştirmesini gerçek vanilla server üzerinde sınar. Amaç yalnızca `use_item_on` ACK'ini görmek değil, komşu bloğun taş state'ine geçtiğini gözlemlemektir.

## Java referansı

`GameProtocols.java` serverbound dispatch sırasındaki `SET_CARRIED_ITEM` ve `SET_CREATIVE_MODE_SLOT` kimlikleri sırasıyla `53` ve `56`'dır. `ServerboundSetCreativeModeSlotPacket` big-endian `short slotNum` ve doğrulanmış optional `ItemStack` taşır. Dolu `ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC` sırası count VarInt'i, item holder registry ID'si ve `DataComponentPatch.DELIMITED_STREAM_CODEC`'un positive/negative sayaçlarıdır.

`Items.java` içinde `AIR` ilk, `STONE` ikinci built-in item kaydıdır. `BuiltInRegistries.ITEM` defaulted registry'si bu statik kayıt sırasını kullandığı için pinned Java 26.2 koşusunda taş item registry ID'si `1`'dir. Item registry configuration paketlerinde gönderilmediğinden bu ID dinamik registry paketlerinden çıkarılmaz.

## Probe girdisi ve akışı

- Yaratıcı slot: `36` (player inventory hotbar slot `0`), stack count `1`, stone item ID `1`
- Boş component patch: positive count `0`, negative count `0`
- Seçili hotbar: `SET_CARRIED_ITEM` ile `short 0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki block position
- Face: `NORTH`, 3D direction ID `2`; click `(0.5, 0.5, 0.0)` f32
- `inside=false`, `worldBorderHit=false`
- Placement sequence: `5`

İlk `player_loaded` sonrasında Rust sırasıyla yaratıcı slot, carried item ve `use_item_on` packet'lerini yollar. Vanilla server sequence `5` için ACK gönderir; `use_item_on` hedefi ve kuzey komşusunu block update ile bildirir. Kontrollü taş platformda komşu state ID'si `1` olduğunda gerçek taş yerleşimi doğrulanır.

## Sınır

Bu dilim genel item component codec'ini, survival tüketimini, ray hit üretimini, client-side block prediction/rollback'u, inventory/container click mantığını ve görsel/ses geri bildirimini kapsamaz. Doğal spawn su veya dolu bir komşu bloğa denk geldiğinde ACK tek başına placement başarısı sayılmaz; probe komşu state'ini ayrıca ölçer.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Spawn aramasının suya veya mevcut yüzeye denk gelmemesi için geçici dünyada `32..168` x/z aralığında y=`100` taş platformu oluşturuldu ve world spawn `(100,101,100)` yapıldı. JAR, dünya ve server çıktıları depoya alınmadı.

```text
cargo run --offline --release --example creative_place_probe -- 127.0.0.1:25566 RustPlace6
cargo run --offline --release --example creative_place_probe -- 127.0.0.1:25566 RustPlace7
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustPlace6 | RustPlace7 |
| --- | ---: | ---: |
| stone item registry ID | `1` | `1` |
| creative slot packets sent | `1` | `1` |
| carried item packets sent | `1` | `1` |
| use item on packets sent | `1` | `1` |
| block changed ACK sequences | `[5]` | `[5]` |
| target block update | `{x:94,y:100,z:103,state_id:1}` | `{x:91,y:100,z:93,state_id:1}` |
| north neighbor update | `{x:94,y:100,z:102,state_id:1}` | `{x:91,y:100,z:92,state_id:1}` |
| chunk packet | 24 | 24 |
| decoded chunk | 24 | 24 |
| chunk batch finished | 6 | 6 |
| received packet | 125 | 125 |

Server logunda RustPlace6 `(94.5,101.0,103.5)`, RustPlace7 `(91.5,101.0,93.5)` konumlarında giriş yaptı; iki hedef de taş platformunun üstündeydi. Her iki koşuda hedef taş state `1` olarak korundu ve `NORTH` komşusu taş state `1` oldu. Vanilla logunda protocol error, timeout veya carried/creative slot warning oluşmadı.

Doğal spawn ile yapılan önceki RustPlace5 denemesinde komşu state `9` kaldı. ACK `[5]` ve iki update alınmasına rağmen bu koşu placement başarısı olarak kaydedilmedi; bu, server ACK'inin item placement sonucunu tek başına kanıtlamadığını gösterdi.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde creative slot `ItemStack` wire sırasını, hotbar seçimini, side `use_item_on` placement'ını, ACK'i ve otoriter taş state update'ini doğrular. Genel item component'leri, survival item tüketimi, ray hit/range üretimi, inventory/container işlemleri, client prediction/rollback ve görsel/ses feedback'i açık kalır.

Sözleşme ayrıntıları [creative placement contract belgesindedir](#sözleşme). Boş el akışı [ayrı kayıttadır](use-item-on-conformance.md); creative block break [ayrı kayıttadır](block-break-conformance.md).
