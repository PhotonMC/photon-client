# T103 — Sunucu kaynak paketi: paketler ve onay ekranı

**Amaç:** `ResourcePackPush`/`Pop` paketleri (yapılandırma ve oyun), `ServerboundResourcePack` durumları ve kullanıcı onay ekranı (kabul/red, zorunlu paket). İndirme ve yükleme T104'te; bu görev paketi **indirmeden** `DECLINED` ya da kabulde T104'e devreder.
**Bağımlılık:** T101. **Boy:** M.

## Oku
- Java: `network/protocol/common/{ClientboundResourcePackPushPacket,ClientboundResourcePackPopPacket,ServerboundResourcePackPacket}.java`, `client/resources/server/ServerPackManager.java`, `client/gui/screens/ConfirmScreen.java` (`ServerPackConfirmScreen` benzeri: `grep -rn "serverPack" client/`).
- Rust: `crates/photon-render/src/gui/screen/confirm.rs`, `crates/photon/src/controls/confirm.rs`.

## Yapılacaklar
1. Paket çözümü; pack listesi (kimlik, adres, özet, zorunluluk, istem metni).
2. Onay ekranı G tarifiyle (`gui` oracle satırı: düzen, düğmeler).
3. Red: `DECLINED` gönderilir, zorunluysa Java'daki gibi bağlantı kesilir.

## Oracle
G (`server_pack_confirm_screen`); paket birim testleri.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
İndirme, doğrulama, yükleme (T104).

## Belgeler
`docs/gui-oracle.md`, kayıt `client-resource-pack-prompt-conformance.md`.

## Devir notu
Yapıldı: push/pop/yanıt paketleri (yapılandırma ve oyun), `PlaySession::{take_pack_requests,answer_resource_pack,disconnect}`, `ConfirmScreen::server_pack` + `Controls::pack_prompt`, `confirm_screen` oracle'ında `pack-*` satırları (64/64 Java parity). Sapma: indirme yok, kabul `ACCEPTED`+`FAILED_DOWNLOAD`; yapılandırmada paket `DECLINED`. Açık: canlı Pumpkin denemesi yapılmadı. Kayıt: [client-resource-pack-prompt-conformance.md](../records/client-resource-pack-prompt-conformance.md).
