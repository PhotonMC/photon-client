# P12 — Blok eylemi sequence ACK sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, ilk dünya yükleme akışından sonra vanilla server'a zararsız bir `ABORT_DESTROY_BLOCK` packet'i gönderir ve sequence ACK'ini okur. Amaç, block action wire alanlarının, oyun oturumundaki serverbound packet ID'sinin ve server'ın `ClientboundBlockChangedAckPacket` cevabının çalıştığını sınamaktır.

## Java referansı

`ServerboundPlayerActionPacket` action enum/BlockPos/direction/sequence sırasıyla, `ServerGamePacketListenerImpl.handlePlayerAction` içindeki loaded kontrolü ve `ackBlockChangesUpTo` çağrısıyla, server tick'inde `ClientboundBlockChangedAckPacket` gönderimiyle birlikte incelendi. `FriendlyByteBuf.writeEnum`, `writeBlockPos` ve Direction'ın `DOWN` wire ID'si de kontrol edildi. Orijinal kaynak depoya kopyalanmadı.

## Probe girdisi ve akışı

- Action: `ABORT_DESTROY_BLOCK` enum ordinal `1`
- Block position: `BlockPos.ZERO`, packed long `0`
- Direction: `DOWN`, ID `0`
- Sequence: `1`

Bu sentinel hedef menzil dışında olduğu için block break state'i veya geçici dünya içeriği değişmez. Java server yine sequence'i ACK kuyruğuna alır; sonraki tick'te `ClientboundBlockChangedAckPacket(sequence=1)` gönderir. Rust trace gönderilen action sayısını ve ACK sequence listesini kaydeder.

## Sınır

Bu dilim hit test, reach, gerçek block state değişimi, kazma ilerlemesi/iptali, placement, item use, prediction/rollback ve client world cache uygulamaz. ACK alınması oynanabilir istemci veya Java action davranışının tamamı değildir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example action_probe -- 127.0.0.1:25566 RustAction3
cargo run --offline --example action_probe -- 127.0.0.1:25566 RustAction4
```

İki bağımsız koşunun çıktısı:

| Gözlem | RustAction3 | RustAction4 |
| --- | ---: | ---: |
| play login | `true` | `true` |
| ilk teleport gözlemi | `[{teleport_id:1, relative_flags:0}]` | `[{teleport_id:1, relative_flags:0}]` |
| player loaded sent | `true` | `true` |
| player action packets sent | 1 | 1 |
| block changed ACK sequences | `[1]` | `[1]` |
| chunk packet | 24 | 24 |
| decoded chunk | 24 | 24 |
| chunk batch finished | 6 | 6 |
| received packet | 378 | 402 |

Her iki koşuda Rust, `ABORT_DESTROY_BLOCK` için action `1`, packed `BlockPos.ZERO`, `DOWN` ve sequence `1` alanlarını gönderdi. Vanilla server aynı sequence için ACK döndürdü; server logunda protocol error, timeout veya action mismatch warning oluşmadı.

## Açık kapsam

Bu sonuç gerçek vanilla server'ın sequence ACK akışını ve block action packet wire sırasını doğrular. Reach/hit sonucu, gerçek block break veya placement, item use, prediction/rollback, client world state, envanter ve görsel/ses feedback'i henüz açık kalır.

Sözleşme ayrıntıları [block action contract belgesindedir](#sözleşme). İlk oyuncu konumu [ayrı kayıtta](player-position-conformance.md), kontrollü hareket [ayrı kayıtta](movement-probe-conformance.md) tutulur. Bu test resmi vanilla sunucusuyla yapıldı; Pumpkin koşusu bu baseline'a karıştırılmadı.
