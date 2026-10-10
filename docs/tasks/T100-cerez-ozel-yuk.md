# T100 — Çerez, özel yük ve sunucu bağlantıları paketleri

**Amaç:** Yapılandırma ve oyun evresinde `CookieRequest`/`StoreCookie`/`ServerboundCookieResponse`, istemciye gelen `CustomPayload`, `CustomReportDetails`, `ServerLinks` ve (yapılandırmada) `ResetChat` paketleri.
**Bağımlılık:** —. **Boy:** M.

## Oku
- Java: `network/protocol/cookie/*`, `network/protocol/common/{ClientboundStoreCookiePacket,ClientboundCustomPayloadPacket,ClientboundCustomReportDetailsPacket,ClientboundServerLinksPacket}.java`, `network/protocol/configuration/ClientboundResetChatPacket.java`, `client/multiplayer/ClientCommonPacketListenerImpl.java` (`handleCookieRequest`, `handleStoreCookie`).
- Rust: `crates/photon-net/src/connection/probe/open.rs` (yapılandırma döngüsü), `connection/play/incoming.rs` (oyun evresi; büyük dosya: `grep -n`), `connection/protocol/ids.rs`.

## Yapılacaklar
1. Paket kodlayıcı/çözücüleri (sınırlar Java'daki gibi: çerez 5120 bayt, anahtar 32767).
2. Bellekte çerez deposu (kimliğe göre); yoksa boş yanıt.
3. `CustomPayload` çözülür, bilinmeyen yük yok sayılır; marka (`minecraft:brand`) saklanır.
4. `ServerLinks` saklanır (`ServerLinksDialogScreen` T106'da kullanır).

## Oracle
Paket çözücü/kodlayıcı birim testleri (Java sınırları); canlı: çerez isteyen bir sunucu yok, kayıt bunu açıkça yazar.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
Çerezin diske yazılması (T102), diyaloglar.

## Belgeler
`docs/records/` altında yeni `client-cookie-payload-conformance.md`, `docs/index.md`.

## Devir notu
(boş)
