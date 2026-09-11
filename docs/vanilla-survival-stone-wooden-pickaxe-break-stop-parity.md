# P12 — Vanilla 26.2 survival wooden pickaxe tool progress doğrulaması

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

Sözleşme ayrıntıları [survival stone wooden pickaxe break stop contract belgesindedir](survival-stone-wooden-pickaxe-break-stop-contract.md). Çıplak el stone sonucu [ayrı kayıttadır](vanilla-survival-stone-break-stop-parity.md); dirt tam kırma [ayrı kayıttadır](vanilla-survival-break-stop-parity.md).
