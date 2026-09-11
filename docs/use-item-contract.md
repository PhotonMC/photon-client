# P12 — Boş elle use item sözleşmesi

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
