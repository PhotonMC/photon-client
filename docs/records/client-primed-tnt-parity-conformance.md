# T40c — Yanan TNT — [client-parity]

Bu kayıt `tnt` entity'sinin (`PrimedTnt`, `TntRenderer`) çizimini ve istemci durumunu kapsar. Kanıt sınıfı `client-parity`'dir: bloğun yerleşimi ve beyaz yanıp sönme bindirmesi gerçek `TntRenderer.submit`'in bıraktığı pose'a ve bindirme koduna karşı karşılaştırılır. Fitil sayacı birim testlidir; canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `TntRenderer.submit`: yarım blok yukarı, `fuse < 10` iken `1 + getSwellAmount(fuse)` ölçeği, `Y(-90)`, `(-0,5, -0,5, 0,5)`, `Y(90)`; blok `submitWhiteSolidBlock` ile (`isLit(fuse)` ise tam beyaz bindirme) çizilir. Fitil `getFuse() − kısmiTik + 1`.
- `PrimedTnt` verisi (`Entity` sekiz alan taşır): `DATA_FUSE_ID` (8, int, varsayılan 80), `DATA_BLOCK_STATE_ID` (9, blok durumu, varsayılan TNT). İstemci tiki fitili bir azaltır; `fuse <= 0` entity'yi siler.
- Işık, `getLightProbePosition` (gözler) ile okunur; TNT minecart'ıyla ortak `getSwellAmount`/`isLit` (T38d).

## Kapsam

- `entity/primed-tnt-pose` 17 satır (fitilin uzun, şişmenin başı ve sonu, her yanıp sönmenin iki yanı, bitmiş ve negatif değerleri, uzak konum); pose ve bindirme kodu karşılaştırılır.
- Rust: `photon-render::minecart::primed_tnt_pose`, `photon-world::primed_tnt` (veri ve sayaç), oyunda `minecarts::remote_primed_tnt` (blok durumu, T34d'nin entity üstü blok çizimiyle).

## Sapmalar ve açık kalanlar

- Sayacın gerçek `PrimedTnt.tick`'e karşı karşılaştırması yok (birim test); konum tahmini önceki `entity-primed-tnt-tick` kaydındadır.
- Blok durumu kaydından başka bir durum çizilmeye çalışılırsa entity üstü blok yolunun sınırları geçerlidir (T34d).
- Partikül (`SMOKE`) ve patlama yok.
