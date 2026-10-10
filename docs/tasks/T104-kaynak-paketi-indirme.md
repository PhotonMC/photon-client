# T104 — Sunucu kaynak paketi: indirme, doğrulama, yükleme

**Amaç:** Kabul edilen paketin indirilmesi (özet doğrulaması), geçici dizinde saklanması, kaynak yığınına eklenip varlıkların yeniden yüklenmesi ve sunucuya durum bildirimi.
**Bağımlılık:** T103 ve **kullanıcı kararı** (HTTP/TLS istemcisi bağımlılığı; `P04.c` varlık yeniden yükleme akışı). **Boy:** L.

## Oku
- Java: `client/resources/server/{DownloadedPackSource,PackDownloader,PackLoadFeedback,PackReloadConfig}.java`.
- Rust: `crates/photon/src/assets.rs`, `docs/asset-pack.md` varsa (`grep -rn "P04" docs/roadmap.md`).

## Yapılacaklar
Bağımlılık kararı alındıktan sonra bölünür: indirme+özet / yığın ve yeniden yükleme / durum bildirimi.

## Oracle
Özet doğrulaması birim testleri; yerel HTTP sunucusuyla indirme.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
—

## Belgeler
Karar kaydı `docs/` altında yeni bir belge olarak (bkz. `audio-backend-decision.md`).

## Devir notu
(boş)
