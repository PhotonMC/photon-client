# T83 — Hasar sırasında kameranın eğilmesi — [client-parity]

Gerçek `GameRenderer.bobHurt` (Java, `GameRenderer` ve `CameraRenderState` ile, yansımayla çağrılır) ile Rust `voxel_render::camera::apply_hurt` aynı girdide bit bit karşılaştırılır (`entity/camera-hurt`, 15 satır: akış bitmiş, akış boyunca altı nokta ve yön, daha zayıf ve sıfır eğim seçeneği, uzun akış, ölüm, ölürken akış).

## Java kaynak incelemesi

`bobHurt`: yaşayan kamera varlığı ölüyorsa görüntü `Z` ekseninde `40 − 8000/(min(deathTime, 20) + 200)` derece yatar; `hurtTime` (kare kesri düşülmüş) eksiye inmediyse `hurt = sin((hurtTime/hurtDuration)^4·π)` hesaplanır, görüntü hasarın geldiği yönde (`−hurtDir` `Y`, `−hurt·14·damageTiltStrength` `Z`, `hurtDir` `Y`) eğilir. `Camera.extractRenderState`: `hurtTime − kare kesri`, `deathTime + kare kesri`, `Player.getHurtDir`. `Player.animateHurt(yaw)` `hurtDir`'i `HurtAnimation` paketinin yaw'ına atar. `renderLevel` önce `bobHurt`'ü sonra `bobView`'u aynı yığına uygular ve izdüşüme çarpar; elin pozu da (`renderItemInHand`) aynı sırayla.

## Yapılan

- `camera::{HurtView, apply_hurt, view_hurt}`; `LocalPlayer::{animate_hurt, hurt_view}` (`hurtDuration`, `hurtDir`, `deathTime` sayacı); `ClientboundHurtAnimationPacket`'in yaw'ı yerel oyuncuya; oyun görüntüsü ve gökyüzü izdüşümü ve ilk şahıs eli bu eğimle.

## Sapmalar ve açık kalanlar

- `damageTiltStrength` seçeneği `options.txt`'e yazılmaz (Java'da da yazılmıyor) ve ayar ekranı yok: sabit 1,0.
- Hasar kırmızısı ekran örtüsü, gece görüşü, `Darkness`, körlük sisi ve yanma/su altı örtüleri [T83b](../tasks/T83b-ekran-orturleri.md)'de.
- Canlıda bakılmadı (eğimin yönü ve süresi Java kaynağından yazıldı).
