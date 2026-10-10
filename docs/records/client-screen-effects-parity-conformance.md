# Ekran örtüleri (`ScreenEffectRenderer`) — [client-parity]

Kapsam: gözlerin içinde bulunduğu blok (`getViewBlockingState`: sekiz nokta, `isViewBlocking`,
görünmez blok hariç), bloğun, suyun ve ateşin dörtgenleri (`submitBlockSprite`, `submitWater`,
`submitFire`: köşeler, doku koordinatı, renk) ve oyunda çizimleri.

Kanıt: `movement/local-player-ticks` satırları (`buried-in-*`) gerçek `LocalPlayer`'ın her tikte
seçtiği bloğu (`view_block`) Rust'la bit bit karşılaştırır (taş, cam, bariyer, ruh kumu);
`entity/screen-effect-quads` satırları gerçek `ScreenEffectRenderer`'ın blok, su ve ateş
dörtgenlerini (konum, doku koordinatı, renk) sahte sprite köşeleriyle karşılaştırır.

Oyunda: dörtgenler elin görünüm uzayında, elin izdüşümüyle, ışık ve sis olmadan karışarak
(`ScreenEffect` boru hattı) çizilir; ateş ve su canlıda görüldü (`.data/runs/shots/screen_effect_*.png`),
gömülü blok görüntülenemedi (oyuncu dışarı itildi).

## Sapmalar (açık)

- Su örtüsünün parlaklığı gökyüzü kararmasını (`gameplay/sky_light_level`) taşımaz.
- Yerel oyuncunun ölçeği (`getScale`) bire sabittir; öznitelikten okunmaz.
- Ateş durumu sunucunun `SharedFlags` verisinden gelir (`remainingFireTicks` yok).
- Eşya etkinleştirme animasyonu (totem), körlük/karanlık/gece görüşü sisi ve portal dönmesi yok.
