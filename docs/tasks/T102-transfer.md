# T102 — Transfer paketi ve çerezlerin saklanması

**Amaç:** `ClientboundTransferPacket`: bağlantıyı kapatıp verilen adrese `TRANSFER` niyetiyle yeniden bağlanma, çerezlerin bu geçişte taşınması.
**Bağımlılık:** T100, T101. **Boy:** M.

## Oku
- Java: `network/protocol/common/ClientboundTransferPacket.java`, `client/multiplayer/ClientCommonPacketListenerImpl.java` (`handleTransfer`), `client/multiplayer/TransferState.java`, `client/gui/screens/ConnectScreen.java`.
- Rust: `crates/photon-net/src/handshake.rs` (niyet), `crates/photon/src/menu.rs` (bağlanma akışı).

## Yapılacaklar
1. Handshake niyeti `TRANSFER`.
2. Çerez deposu `TransferState` gibi taşınır (yalnız aktarımda).
3. Bağlanma ekranı yeniden açılır.

## Oracle
Birim testler (niyet, çerez taşıma).

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
Çerezlerin diskte kalıcılığı (Java'da da yok).

## Belgeler
Kayıt `client-transfer-conformance.md`, `docs/index.md`.

## Devir notu
Yapıldı: `ClientboundTransferPacket` (yapılandırma ve oyun), `ConnectionError::Transfer`, `PlaySession::connect_through` (aktarım döngüsü, ad çözümü, `TRANSFER` niyeti ve el sıkışmada sunucu adı), çerezler taşınır. Sahte sunucu testi geçti; gerçek aktarım denenmedi. Sapma: bloklayarak bağlanır, `SRV` yok. Kayıt: [client-transfer-conformance.md](../records/client-transfer-conformance.md).
