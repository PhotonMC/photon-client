# T101 — Yeniden yapılandırma (oyun → yapılandırma → oyun)

**Amaç:** `ClientboundStartConfigurationPacket` ile oyunun yapılandırmaya dönmesi, onay paketi, yapılandırma döngüsünün açılış ve yeniden yapılandırmada ortak kullanımı ve ikinci `Login` ile dünyanın sıfırlanması.
**Bağımlılık:** —. **Boy:** M.

## Oku
- Java: `client/multiplayer/ClientPacketListener.java` (`handleConfigurationStart`), `ClientConfigurationPacketListenerImpl.java`, `network/protocol/game/{ClientboundStartConfigurationPacket,ServerboundConfigurationAcknowledgedPacket}.java`.
- Rust: `crates/photon-net/src/connection/probe/open.rs` (tek seferlik yapılandırma), `connection/play/lifecycle.rs`, `connection/session.rs`.

## Yapılacaklar
1. `open.rs`'deki yapılandırma döngüsünü yeniden kullanılabilir bir aşamaya çıkar (açılışta ve oyun içinden).
2. `StartConfiguration` gelince oyun durumu bırakılır (ekranlar kapanır, dünya silinir), onay gönderilir, yapılandırma, sonra yeni oyun.
3. Kayıtlar/etiketler yeni yapılandırmayla değişir.

## Oracle
Paket testleri; canlı: sunucuda yeniden yapılandırma tetikleyen bir komut yoksa Pumpkin'de elle (kayıt boşluğu yazılır).

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
Transfer (T102), kaynak paketi.

## Belgeler
Aynı kayıt veya `docs/records/client-reconfiguration-conformance.md`, `docs/index.md`.

## Devir notu
(boş)
