# P12 — Survival oak planks wooden axe tool progress sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun hotbar slot 0'daki wooden axe'ı seçip oak planks üzerinde `START_DESTROY_BLOCK` göndermesini, 1.2 saniye sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç wooden axe'ın doğru `mineable/axe` tool hızını oak planks sertliğiyle birlikte doğrulamak ve kırma sonrası dayanıklılık bileşeni yayınını gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de oak planks `strength(2.0F, 3.0F)` ile kayıtlıdır. Wooden axe `ToolMaterial.WOOD` kullanır; wooden material'ın mining speed'i `2.0F`, dayanıklılığı `59` ve axe kaydı `Items.java` içindedir. Tool material, eşleşen `mineable/axe` blokları için `TOOL` component'ine bu hızı ve blok başına `1` hasarı yazar; aynı item `WEAPON` component'ini de taşır.

`BlockBehaviour` doğru tool için destroy progress modifier'ını `30` seçer. Server game mode, geçen tick sayısıyla progress'i çarpar ve `0.7F` stop eşiği geçilince hedefi kırar. Kırma dayanıklılık değişimi, wooden axe slotuna `DAMAGE` component patch'i olarak yayınlanır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:wooden_axe`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` oak planks platformu
- Direction: `UP`, ID `1`
- Start sequence: `32`
- Stop sequence: `33`
- Start ile stop arasında bekleme: `1_200 ms`

Rust probe `SET_CARRIED_ITEM` ve iki action paketini yollar; ACK, slot ve block update listelerini toplar. Beklenen sonuç carried item packet sayısı `1`, action sayısı `2`, ACK `[32,33]`, hazırlık slotunda item registry `942` ve boş patch, bitiş slotunda aynı item için `DAMAGE` component ID `3`, ayrıca hedefte state `0` block update'idir.

## Sınır

Bu dilim yalnızca wooden axe ile oak planks kırmayı ve tek blok dayanıklılık güncellemesini sınar. Farklı tool/block kombinasyonları, tool dayanıklılığının birden fazla blokta ilerlemesi, enchantment, erken stop, tick başına destruction progress, client prediction ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` oak planks platformu ve `(100,101,100)` world spawn'ı kullanılarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:wooden_axe` uygulandı; geçerli koşular yalnızca bu komutun başarıyla loglandığı oturumlar oldu.

```text
cargo run --offline --release --example survival_oak_planks_wooden_axe_break_stop_probe -- 127.0.0.1:25566 RustOakAxe1
cargo run --offline --release --example survival_oak_planks_wooden_axe_break_stop_probe -- 127.0.0.1:25566 RustOakAxe2
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustOakAxe1 | RustOakAxe2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| player action packets sent | `2` | `2` |
| container preparation | `state=2, slot=36, count=1, item=942, components=[]` | `state=2, slot=36, count=1, item=942, components=[]` |
| container break update | `state=3, slot=36, count=1, item=942, components=[3]` | `state=3, slot=36, count=1, item=942, components=[3]` |
| block changed ACK sequences | `[32,33]` | `[32,33]` |
| block update | `(98,100,102,state=0)` | `(91,100,95,state=0)` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `1106` | `1773` |

Server logu RustOakAxe1 için `(98.5,101.0,102.5)`, RustOakAxe2 için `(91.5,101.0,95.5)` konumlarını ve her iki oyuncuya wooden axe slot değişikliğinin başarıyla uygulandığını gösterdi. Java floor hedefleri sırasıyla `(98,100,102)` ve `(91,100,95)` oldu. İki probe da hedef için state `0` authoritative block update'i aldı; server console kontrolleri hedeflerin air olduğunu ayrıca doğruladı. Vanilla logunda geçerli koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç gerçek vanilla server üzerinde wooden axe seçiminin oak planks hedefinde 1.2 saniyelik kontrollü aralıkta kırma eşiğini geçtiğini, carried item paketini, `DAMAGE` component'li slot yayını, action ACK sırasını ve authoritative air update'ini iki geçerli oturumda doğrular. Farklı hardness/tool kombinasyonları, çoklu dayanıklılık, enchantment, erken stop, client prediction ve inventory click açık kalır.

Sözleşme ayrıntıları [survival oak planks wooden axe break stop contract belgesindedir](#sözleşme). Wooden shovel sonucu [ayrı kayıttadır](survival-dirt-wooden-shovel-break-stop-conformance.md); çıplak el oak planks sonucu [ayrı kayıttadır](survival-oak-planks-break-stop-conformance.md).
