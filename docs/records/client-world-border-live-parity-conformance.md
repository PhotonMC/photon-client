# T84 — Dünya sınırı canlı oturumda: paketler ve oyuncunun çarpışması — [client-parity] (çarpışma) · [server-conformance] (paketler)

Dünya sınırı durumu ve şekli [önceki kayıtlarda](client-world-border-state-conformance.md) vardı; bu kayıt onu canlı oyuna bağlar.

## Java kaynak incelemesi

- `ClientPacketListener.handleInitializeBorder` ve `handleSetBorder*`: merkez, boyut (`lerp` süresi sıfırdan büyükse `lerpSizeBetween`, değilse `setSize`), en büyük boyut, uyarı süresi ve uzaklığı. Paketler: `InitializeBorder` (iki çift merkez, iki çift boyut, var long süre, üç var int), `SetBorderCenter`, `SetBorderLerpSize` (var long süre), `SetBorderSize`, `SetBorderWarningDelay`, `SetBorderWarningDistance`.
- `Entity.collectColliders` (`collectAllColliders`): kaynak varlık sınırın yakınındaysa (`WorldBorder.isInsideCloseToBorder`) sınırın çarpışma şekli blok çarpışmalarına eklenir; `CollisionGetter.noCollision(entity, kutu)` da sınırı sayar.

## Yapılan

- Altı paketin ayrıştırması (`protocol_tests`) ve `ClientWorld`'e uygulanması; sınır her istemci tikinde zaten ilerliyordu.
- `ClientWorld::border_collider(kaynak, kutu)`; `Level::{colliders, no_collision}` kaynağın konumunu alır, sınır şeklini başa ekler. Yerel oyuncu, binekler ve yerdeki eşya bununla çarpışır; otomatik zıplamanın kutu taraması yalnız bloklara bakar (Java'daki gibi).
- Kanıt (`client-parity`): `movement/local-player-ticks` sahnelerine `border` alanı (merkez, boyut) ve beş satır: sınıra yürüme, koşarak zıplama, köşede kayma, dışarıdan başlama, eğilerek yürüme; gerçek `LocalPlayer` ile bit bit eşit.

## Sapmalar ve açık kalanlar

- Sınırın çizimi (`WorldBorderRenderer`) ve sınır dışındayken hasar kırmızısı yok: [T84b](../tasks/T84b-sinir-cizimi.md).
- `getDamagePerBlock`, `safeZone` paketlerle gelmez (sunucu hasar verir); istemcide okunmaz.
- Sınırı bekleyen sunucu-uyum kaydı yok; canlıda bakılmadı.
