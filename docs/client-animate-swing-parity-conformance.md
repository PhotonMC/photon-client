# T43 — `Animate` paketi, kol sallaması ve `EntityEvent` durumları — [client-parity]

Bu kayıt, diğer canlıların kol sallamasını (`ClientboundAnimatePacket`,
`LivingEntity.swing`/`updateSwingTime`/`getAttackAnim`) ve `EntityEvent` ile açılan
koyun yemesini ve tavuğun kanat çırpmasını Java 26.2 ile aynı değerlerle
sürmeyi kapsar. Kanıt sınıfı `client-parity`'dir: `movement/living-entity-ticks` ailesi
gerçek `ClientPacketListener` işleyicilerine (`handleAnimate`, `handleEntityEvent`,
`handleMoveEntity`) gerçek bir `LivingEntity` ile paket gönderir, `level.tickNonPassenger`
ile tikler ve her tikte alanları bit bit karşılaştırır (tavuğun gerçek `aiStep`'i dahil).

## Java kaynak incelemesi

- `ClientPacketListener.handleAnimate`: eylem 0 ana el, 3 yan el sallar (`swing(hand)`);
  2 uyanma (oyuncu), 4 ve 5 parçacık yayıcı (parçacık motoru olmadan yapılmaz).
- `LivingEntity.swing`: sallama yoksa, ya da süren sallamanın yarısı geçmişse (`swingTime`
  süre/2 veya −1) yeni sallama başlar (`swingTime = −1`, kol kaydedilir).
  `updateSwingTime`: her tikte `swingTime++`, süreye varınca sıfırlanır; `attackAnim =
  swingTime / süre`. Bu yöntemi yalnız `Player.aiStep`, `Monster.aiStep` ve `Mannequin`
  çağırır: oyuncu/canavar dışı canlıda (domuz) sallama başlar ama ilerlemez.
  `baseTick` her tik önceki `attackAnim`'i saklar; `getAttackAnim` iki tik arasını
  yeni sallamada sarılarak karıştırır.
- `ArmedEntityRenderState`: `attackArm` ana kol, yan el sallıyorsa karşısı; `attackTime`
  `getAttackAnim`.
- `Sheep.handleEntityEvent` 10 yeme zamanlayıcısını 40 yapar, istemci `aiStep` azaltır;
  `Chicken.aiStep` kanat durumunu sürer (`flapping` 1'den başlar).

## Veri

`monster` (türün `Monster` olması) türün katalog satırına eklendi
(`movement/entity-pick-table`, `tools/generate/entity_picks.py`): oracle her türü yaratıp
`instanceof Monster` sorar, tablo her koşuda baştan karşılaştırılır.

## Kapsam ve dal kararı

`movement/living-entity-animations` 14 satır: zombi, yan el, sallama sırasında yeni sallama
(erken ve yarıdan sonra), iskelet ve piglin (canavar), domuz (canavar değil), sessiz; koyun
yemesi (tek ve iki kez); tavuk havada, inip kalkarak. Tür içi örnekler tablo satırıdır.

## Sapmalar (açık)

- Sallama süresi sabit altı (`SwingAnimation.DEFAULT`): tutulan eşyanın `SWING_ANIMATION`
  bileşeni (mızrak) ve acele/madenci yorgunluğu etkileri başkaları için okunmuyor (yerel
  oyuncu için de öyle).
- `Animate` eylemleri 2/4/5 (uyanma, kritik vuruş parçacıkları) uygulanmıyor.
- `KeyframeAnimation` ve `AnimationState` oynatıcısı bu görevden ayrıldı (T43b): ilk
  tüketicisi (deve, aksolotl…) gelince yazılır.
