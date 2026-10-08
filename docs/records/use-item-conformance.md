# P12 — Boş elle use item sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, istemci yüklendikten sonra açık alanda boş ana elle `ServerboundUseItemPacket` gönderilmesini sınar. Amaç packet alan sırasını, sequence ACK'ini ve boş stack'in dünyada yan etki üretmemesini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`ServerboundUseItemPacket` alanları `InteractionHand`, sequence VarInt'i, yRot f32 ve xRot f32 sırasındadır. `MAIN_HAND` enum ordinal'ı `0`, `use_item` play serverbound packet ID'si protokol 776'da `67`'dir. Java server, client loaded durumunda sequence'i ACK kuyruğuna alır; oyuncunun elindeki stack boşsa item kullanımına ve rotation snap'ine girmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- El: `MAIN_HAND`, ordinal `0`
- Sequence: `10`
- yRot: `0.0f`
- xRot: `0.0f`
- Ön koşul: configuration tamamlanmış, ilk chunk alınmış ve `player_loaded` gönderilmiş oturum

Rust probe bu tek paketi gönderir ve kısa drain süresinde gelen ACK ile block update paketlerini kaydeder. Beklenen gözlem `block_changed_ack_sequences=[10]`, `use_item_packets_sent=1` ve boş block update listesidir.

## Sınır

Boş el sonucu item davranışını doğrulamaz. Dolu stack kullanımı, tüketim/component değişimi, iki el önceliği, server swing'i, rotation değişimi ve istemci prediction sonraki P12 dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Probe, ilk chunk ve `player_loaded` sonrasında boş ana elle açık alana `use_item` gönderdi.

```text
cargo run --offline --release --example use_item_probe -- 127.0.0.1:25566 RustUse5
cargo run --offline --release --example use_item_probe -- 127.0.0.1:25566 RustUse6
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustUse5 | RustUse6 |
| --- | ---: | ---: |
| use item packets sent | `1` | `1` |
| block changed ACK sequences | `[10]` | `[10]` |
| block update observations | `[]` | `[]` |
| chunk packet | `24` | `24` |
| decoded chunk | `24` | `24` |
| chunk batch finished | `6` | `6` |
| received packet | `101` | `102` |

Vanilla server, boş stack nedeniyle item action veya rotation snap sonucu üretmedi; block update alınmadı. Server logunda protocol error, timeout veya `UseItemPacket` rejection warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç `use_item` packet ID `67`, Java alan sırası, loaded oturum ACK'i ve boş ana elin dünya üzerinde değişiklik yapmamasını iki vanilla oturumunda doğrular. Dolu stack item kullanımı, tüketim/component güncellemeleri, iki el önceliği, swing ve client prediction açık kalır.

Sözleşme ayrıntıları [use item contract belgesindedir](#sözleşme). Boş elle blok yüzüne kullanım [ayrı kayıttadır](use-item-on-conformance.md); creative placement [ayrı kayıttadır](creative-placement-conformance.md).
