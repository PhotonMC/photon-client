# P12 — Yaratıcı item stack ile blok yerleştirme sözleşmesi

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
