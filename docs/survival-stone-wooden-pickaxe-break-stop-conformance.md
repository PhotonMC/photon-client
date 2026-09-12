# P12 — Survival wooden pickaxe tool progress sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun hotbar slot 0'daki wooden pickaxe'ı seçip stone üzerinde `START_DESTROY_BLOCK` göndermesini, yeterli tick geçtikten sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini sınar. Amaç çıplak el stone probe'undan farklı olarak doğru tool'un destroy progress hesabına etkisini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

Java 26.2'de stone `strength(1.5F, 6.0F)` ve wooden pickaxe `ToolMaterial.WOOD` üzerinden pickaxe özellikleriyle kayıtlıdır. `BlockBehaviour` doğru tool için modifier `30` kullanır ve oyuncunun tool destroy speed'ini stone destroy speed'ine böler. Server game mode progress'i `0.7F` eşiğine ulaştığında hedefi kırar ve state `0` block update'i yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:wooden_pickaxe`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` stone platformu
- Direction: `UP`, ID `1`
- Start sequence: `20`
- Stop sequence: `21`
- Start ile stop arasında bekleme: `2200 ms`

Rust probe `SET_CARRIED_ITEM` ve iki action paketini yollar; ACK, destruction progress ve block update listelerini toplar. Beklenen sonuç carried item packet sayısı `1`, ACK `[20,21]`, destruction listesi boş ve hedefte state `0` block update'idir.

## Sınır

Bu dilim yalnızca server tarafından hotbar'a yerleştirilen wooden pickaxe ile stone kırma süresini sınar. Tool dayanıklılığı, enchantment, farklı tool/block kombinasyonları, client prediction, inventory slot yayınları ve tick başına progress sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` stone platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:wooden_pickaxe` uygulandı; geçerli koşular yalnızca bu komutun başarıyla loglandığı oturumlar oldu.

```text
cargo run --offline --release --example survival_stone_wooden_pickaxe_break_stop_probe -- 127.0.0.1:25566 RustPickStop4
cargo run --offline --release --example survival_stone_wooden_pickaxe_break_stop_probe -- 127.0.0.1:25566 RustPickStop5
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustPickStop4 | RustPickStop5 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[20,21]` | `[20,21]` |
| block destruction observations | `[]` | `[]` |
| block update observations | `(95,100,95,state=0)` | `(93,100,99,state=0)` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `1342` | `660` |

Server logu RustPickStop4 için `(95.5,101.0,95.5)` ve RustPickStop5 için `(93.5,101.0,99.5)` konumlarını, her iki oyuncuya wooden pickaxe slot değişikliğinin başarıyla uygulandığını ve bağlantıların temiz kapandığını gösterdi. Java floor hedefleri sırasıyla `(95,100,95)` ve `(93,100,99)` oldu. Console `execute if block` kontrolü RustPickStop5 hedefinin air olduğunu doğruladı; RustPickStop4 için aynı state `0` block update'i alındı ve sonraki platform yenilemesinde hedef yeniden stone yapıldı. Vanilla logunda geçerli koşular için protocol error, timeout veya action warning oluşmadı. Hazırlık komutu geç işlendiği için tamamlanamayan önceki denemeler sonuç tablosuna dahil edilmedi.

## Sonuç ve açık kapsam

Bu sonuç vanilla survival akışında wooden pickaxe seçiminin stone hedefini 2200 ms'lik kontrollü aralıkta kıracak progress oluşturduğunu, carried item paketini, action ACK sırasını ve authoritative air update'ini iki geçerli oturumda doğrular. Tool dayanıklılığı, enchantment, farklı tool/block kombinasyonları, client prediction, inventory slot yayınları ve tick başına progress açık kalır.

Sözleşme ayrıntıları [survival stone wooden pickaxe break stop contract belgesindedir](#sözleşme). Çıplak el stone sonucu [ayrı kayıttadır](survival-stone-break-stop-conformance.md); dirt tam kırma [ayrı kayıttadır](survival-break-stop-conformance.md).
