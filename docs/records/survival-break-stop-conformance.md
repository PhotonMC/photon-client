# P12 — Survival block-break başlatma ve tamamlama sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun bir dirt bloğu için `START_DESTROY_BLOCK` gönderip kontrollü bir tick aralığından sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini gerçek vanilla server üzerinde sınar. Amaç stop eşiğini geçen kırma sonucunu, sequence ACK'lerini ve otoriter air update'ini gözlemlemektir.

## Java referansı

`ServerboundPlayerActionPacket.Action` içinde `START_DESTROY_BLOCK` ordinal `0`, `STOP_DESTROY_BLOCK` ordinal `2`'dir. Packet ID `41` ve Direction `UP` ID `1` olarak kodlanır. `ServerPlayerGameMode.handleBlockBreakAction`, aynı hedefte geçen tick sayısını ve block destroy progress'ini çarpar; sonuç `0.7F` eşiğini geçerse destruction progress'i `-1` ile temizler ve `destroyAndAck` çağırır. Kırılan block için `ClientboundBlockUpdatePacket` state ID `0` döner.

## Probe girdisi ve akışı

- Game mode: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` dirt platformu
- Direction: `UP`, ID `1`
- Start sequence: `8`
- Stop sequence: `9`
- Start ile stop arasında bekleme: `650 ms`

İlk `player_loaded` sonrasında Rust start packet'ini yollar, 650 ms bekler ve stop packet'ini yollar. Bu süre dirt'in boş elle stop eşiğini geçmesi için seçilmiştir. Vanilla server ACK kuyruğunda önce `[8]`, sonra `[9]` gözlenir; hedef state ID `0` block update ile air olur.

## Sınır

Bu dilim tüm hardness/tool kombinasyonlarını, her tick'teki progress değerlerini, delayed destroy yolunu, client-side break prediction'ı, ray hit üretimini ve başka oyuncuya gönderilen destruction progress paketlerini kapsamaz. Sabit bekleme, genel istemci tick eşitliği yerine bu tek kontrollü server senaryosunun girdisidir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Spawn araması için geçici dünyada y=`100` seviyesinde geniş dirt platformu ve `(100,101,100)` world spawn'ı kullanıldı. JAR, dünya ve server çıktıları depoya alınmadı.

```text
cargo run --offline --release --example survival_break_stop_probe -- 127.0.0.1:25566 RustStop1
cargo run --offline --release --example survival_break_stop_probe -- 127.0.0.1:25566 RustStop2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustStop1 | RustStop2 |
| --- | ---: | ---: |
| player action packets sent | `2` | `2` |
| block changed ACK sequences | `[8, 9]` | `[8, 9]` |
| block destruction observations | `[]` | `[]` |
| block update | `{x:90,y:100,z:102,state_id:0}` | `{x:99,y:100,z:97,state_id:0}` |
| chunk packet | 54 | 54 |
| decoded chunk | 54 | 54 |
| chunk batch finished | 16 | 16 |
| received packet | 163 | 163 |

Server logundaki player konumları RustStop1 için `(90.5,101.0,102.5)`, RustStop2 için `(99.5,101.0,97.5)` idi; hedefler block update konumlarıyla sırasıyla `(90,100,102)` ve `(99,100,97)` oldu. Vanilla server console kontrolleri iki hedefin de stop sonrasında `air` olduğunu doğruladı. Actor oyuncuya destruction progress gönderilmemesi Java server kaynaklarındaki alıcı filtresiyle uyumludur. Vanilla logunda protocol error, timeout veya action warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde survival start/stop action sırasını, stop eşiğini geçen dirt kırılmasını, iki ACK'i ve state `0` authoritative update'ini doğrular. Taş ve araç süreleri, tick bazlı progress, delayed destroy, survival item/tool etkileri, client prediction/rollback, ray hit ve actor dışı progress yayını açık kalır.

Sözleşme ayrıntıları [survival break stop contract belgesindedir](#sözleşme). Start→abort [ayrı kayıttadır](survival-break-abort-conformance.md); creative break ve placement [ayrı kayıtlardadır](block-break-conformance.md), [vanilla creative placement](creative-placement-conformance.md).
