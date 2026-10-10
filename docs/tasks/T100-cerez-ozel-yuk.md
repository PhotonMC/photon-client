# T100 — Çerez, özel yük ve sunucu bağlantıları paketleri

**Amaç:** Yapılandırma ve oyun evresinde `CookieRequest`/`StoreCookie`/`ServerboundCookieResponse`, istemciye gelen `CustomPayload`, `CustomReportDetails` ve giriş evresinde `CustomQuery` paketleri (`ServerLinks` bileşen çözümü ister, T106'ya devredildi).
**Bağımlılık:** —. **Boy:** M.

## Oku
- Java: `network/protocol/cookie/*`, `network/protocol/common/{ClientboundStoreCookiePacket,ClientboundCustomPayloadPacket,ClientboundCustomReportDetailsPacket,ClientboundServerLinksPacket}.java`, `network/protocol/configuration/ClientboundResetChatPacket.java`, `client/multiplayer/ClientCommonPacketListenerImpl.java` (`handleCookieRequest`, `handleStoreCookie`).
- Rust: `crates/photon-net/src/connection/probe/open.rs` (yapılandırma döngüsü), `connection/play/incoming.rs` (oyun evresi; büyük dosya: `grep -n`), `connection/protocol/ids.rs`.

## Yapılacaklar
1. Paket kodlayıcı/çözücüleri (sınırlar Java'daki gibi: çerez 5120 bayt, anahtar 32767).
2. Bellekte çerez deposu (kimliğe göre); yoksa boş yanıt.
3. `CustomPayload` çözülür, bilinmeyen yük yok sayılır; marka (`minecraft:brand`) saklanır.
4. Giriş evresinde `CustomQuery` yanıtsız cevaplanır.

## Oracle
Paket çözücü/kodlayıcı birim testleri (Java sınırları); canlı: çerez isteyen bir sunucu yok, kayıt bunu açıkça yazar.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
Çerezin diske yazılması (T102), diyaloglar.

## Belgeler
`docs/records/` altında yeni `client-cookie-payload-conformance.md`, `docs/index.md`.

## Devir notu
Yapıldı: `photon-net/src/cookie.rs` (paketler, `CookieStore`), giriş/yapılandırma/oyun evrelerinde bağlama (`probe/open.rs`, `play/incoming.rs`), çerez ve marka yeniden yapılandırmada korunur. Test: birim + sahte sunucu; vanilla oturumu bozulmadı. Açık: `ServerLinks` (T106), çerezin transferde taşınması (T102). Kayıt: [client-cookie-payload-conformance.md](../records/client-cookie-payload-conformance.md).
