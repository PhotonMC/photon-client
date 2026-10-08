# P12 — Yaratıcı modda blok kırma ve block update sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, ilk dünya yüklemesinden sonra oyuncunun spawn olduğu konumun altındaki bloğa vanilla block-break akışını gönderir. Amaç, gerçek hedef menzili, creative instant-break yolunu, sequence ACK'ini ve server'ın yeni block state'i clientbound `block_update` ile bildirmesini aynı oturumda gözlemlemektir.

## Java referansı

`ServerboundPlayerActionPacket` alan sırası ve action enum ordinal'ları; `ServerGamePacketListenerImpl.handlePlayerAction` içindeki loaded/ACK akışı; `ServerPlayerGameMode.handleBlockBreakAction` içindeki menzil, creative `destroyAndAck` ve block update davranışı incelendi. Hedef blok Java `BlockPos.containing` floor semantiğiyle oyuncu konumunun bir blok altından türetilir. `ClientboundBlockUpdatePacket` packed `BlockPos` ve block-state registry ID'si taşır.

## Probe girdisi ve akışı

- Action: `START_DESTROY_BLOCK`, enum ordinal `0`
- Hedef: ilk player-position paketindeki `(x, y - 1, z)` konumunun Java floor karşılığı
- Direction: `UP`, ID `1`
- Sequence: `2`
- Server ayarı: creative, spawn protection `0`

İlk `player_loaded` bildirimi gönderildikten sonra Rust tek bir start-destroy packet'i yollar. Vanilla server creative modda hedefi anında air state'ine geçirir, sequence'i ACK kuyruğuna alır ve hedef için state ID `0` olan `block_update` paketini gönderir. Rust trace hedef konumu, state ID'si ve ACK sequence'ini kaydeder.

## Sınır

Bu dilim survival kazma süresi, kazma ilerleme/iptal, ray/hit seçimi, placement, item use, prediction/rollback, client world cache ve görsel/ses feedback'ini uygulamaz. Hedefin spawn altındaki blok olarak seçilmesi kontrollü server probe'udur; genel oyuncu hedefleme algoritması değildir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak1
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak2
cargo run --offline --example block_break_probe -- 127.0.0.1:25566 RustBreak3
```

Üç bağımsız koşunun sonuçları:

| Gözlem | RustBreak1 | RustBreak2 | RustBreak3 |
| --- | ---: | ---: |
| play login | `true` | `true` | `true` |
| player loaded sent | `true` | `true` | `true` |
| player action packets sent | 1 | 1 | 1 |
| block changed ACK sequences | `[2]` | `[2]` | `[2]` |
| block update observations | `[{x:24,y:76,z:-30,state_id:0}]` | `[{x:25,y:66,z:-39,state_id:0}]` | `[{x:27,y:71,z:-36,state_id:0}]` |
| chunk packet | 21 | 21 | 22 |
| decoded chunk | 21 | 21 | 22 |
| chunk batch finished | 5 | 5 | 5 |
| received packet | 406 | 393 | 322 |

Her üç koşuda `block_update` konumu, server logundaki spawn player konumunun bir blok altındaki hedefle aynıydı; server creative destroy sonrası state ID `0` döndürdü. Vanilla server logunda offline-mode uyarıları dışında protocol error, timeout veya block action warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server üzerinde hedef menzil kontrolünden geçen creative instant-break, sequence ACK ve block update dönüşünü doğrular. Survival destroy progress, abort/stop sırası, ray hit sonucu, placement, item use, client world state, prediction/rollback, inventory ve görsel/ses feedback'i açık kalır.

Sözleşme ayrıntıları [block break contract belgesindedir](#sözleşme). Önceki harmless ACK probe'u [ayrı kayıttadır](block-action-conformance.md). Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
