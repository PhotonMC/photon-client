# P13.b — Eşya bileşen yamaları — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity` (oracle; her koşuda karşılaştırılır)

## Ne

`crates/voxel-game/src/item_components.rs` bir eşyanın bileşen yamasını
(`DataComponentPatch`) referansın ağ kodlayıcılarının birebir portuyla okur:
her bileşen türünün `StreamCodec`'i bir atlama olarak taşındı (yamadaki 111
türün hepsi). Okumanın iki işi var:

- Sunucudan gelen bir yığının nerede bittiğini bulmak. Eskiden yalnız 20 tür
  biliniyordu; bilinmeyen bir bileşen paketin tamamını okunamaz yapıyordu.
- İstemcinin sunucuya yığın gönderirken kullandığı biçimi yazmak:
  `DELIMITED_STREAM_CODEC` her değerin önüne uzunluğunu koyar. Yaratıcı
  envanter tablodan bir yığın alıp hotbar'a koyduğunda `SetCreativeModeSlot`
  bu biçimle gider; uzunluksuz yazılan tablo (tablo, sunucudan gelen biçimde
  saklanır) sunucuda `DecoderException` ile bağlantıyı kesiyordu (tablo,
  ok, havai fişek, kitap alınca oyunun kapanması).

Ağ kodlayıcısı olmayan, yalnız kalıcı kodlayıcısı olan bileşenler
(`custom_data`, `map_decorations`, `recipes`, `lock` gibi) kalıcı
kodlayıcının ürettiği etiket olarak gider; port onları etiket sayar.

## Parity

`block/item-components` (BlockOracle) her eşyanın varsayılan bileşenlerini
yama olarak iki biçimde yazar: sunucu biçimi ve uzunluklu istemci biçimi.
`tools/generate/item_components.py` sunucu biçimini
`crates/voxel-data/src/item/components.rs`e yazar (1537 eşya, kayıt sırasında). Adaptör
tablodaki her yamayı okur ve iki biçimi yeniden yazar; referansın baytlarıyla
ve bileşen numaralarıyla eşleşir (`block` ailesi `parity_passed`). Yaratıcı
sekme tablosunun 1896 yığını da birim testinde tam okunur.

## Değerler

`crates/voxel-game/src/item_values.rs` bir yığının bileşen değerlerini yamadan, yama yoksa
eşyanın varsayılanından (yukarıdaki tablo) okur; yalnız çizimin okuduğu
türler çözülür: `dyed_color`, `map_color`, `firework_explosion`,
`potion_contents`. `PotionContents.getColorOr` portu için
`block/potions` (BlockOracle) mob efektlerini (renk, açıklama anahtarı) ve
iksirleri (ad, efektler) kayıt numarasıyla yazar; `crates/voxel-data/src/item/potions.rs`
(`tools/generate/potions.py`) her koşuda karşılaştırılır.

## Açıklar

- Bileşen predicate'lerinin kısmi (`partial`) kısmı okunmaz
  (`can_place_on` / `can_break` içinde, komutla verilen eşyalarda): bu yama
  okunamaz sayılır ve o yığın paketi hata verir.
- Okunamayan bir yama istemciden giderken atılır (yığın bileşensiz gider),
  yanlış yazılmaz.
- Metin bileşenleri (özel ad, açıklama) ve büyüler çözülmüyor, yalnız
  atlanıyor: ipucu (tooltip) için değerlerin okunması ayrı iştir.
