# T37a — Slime ve magma cube — [client-parity]

Bu kayıt, slime ve magma cube'un modelinin (magma cube'un dilim kaldırması), çizim ölçeğinin
(boyut ve squish), istemci tikinin (inip kalkışta squash/stretch), boyut verisinin ve boyutla
ölçeklenen kutusunun Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir:
model gerçek `MagmaCubeModel`'e, ölçek gerçek `SlimeRenderer`/`AbstractCubeMobRenderer.scale`'e,
tik gerçek `Slime` ve `MagmaCube` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `AbstractCubeMob.tick`: her tik `oSquish = squish`, `squish += (hedef - squish)·0,5`. Yere inince
  (`onGround` ve önceki tikte değil) hedef `-0,5`, yerden kalkınca `1,0`; sonra `decreaseSquish`:
  slime `hedef·0,6`, magma cube `hedef·0,9`. İnişin sesi ve parçacıkları yapılmaz.
  `onGround` istemcide yalnız hareket ve konum paketlerinden gelir.
- `ID_SIZE` (veri 17, int, varsayılan 1): her alındığında (aynı değer de) `onSyncedDataUpdated`
  kutuyu yeniler, `yRot` ve `yBodyRot`'u baş dönüşüne eşitler. Kutu `EntityType` boyutunun boyutla
  çarpılmışıdır (slime `0,52`, magma cube `0,52` temel; boyut 127'ye dek).
- `AbstractCubeMobRenderer.scale`: `w = 1 / (squish/(boyut·0,5 + 1) + 1)`, ölçek `(w·boyut, boyut/w, w·boyut)`;
  slime önce `0,999` ölçekler ve `0,001` yukarı çeker (`downscaleSlightly`). Bu, `scale` kancasıdır
  (çevirme/`-1,501` kaydırması ondan sonra, ölçeklenmiş uzayda).
- `MagmaCubeModel.setupAnim`: `max(0, squish)`; `i`'inci dilim `y = -(4 - i)·squish·1,7`.
- `SlimeOuterLayer`: dış gövde `entityTranslucent` ile, ana gövdeden sonra; `RenderTypes.entityTranslucent`
  `sortOnUpload` ister: dörtgenler kamerada uzaktan yakına (`DISTANCE_TO_ORIGIN`, merkezi ilk ve
  üçüncü köşenin orta noktası) sıralanır. Magma cube ışığı blok ışığı 15 (`getBlockLightLevel`).
- Gölge yarıçapı `boyut·0,25`.

## Kapsam

`entity/magma-cube-model` 6 satır (squish `-0,5…1,7`); `entity/cube-mob-rotations` 33 satır
(slime/magma × boyut 1, 2, 4, 127 × squish; ölen slime; gerçek `scale` kancası dâhil tam poz);
`living-entity-animations`'a 6 satır (iniş, sıçrayış, magma cube'un yavaş sönmesi, baş dönüşünden sonra
boyut verisi, aynı boyutun tekrarı, kutu). Oracle'a `head_rotation` ve `cube_size` olayları ve
`cube` ayrıntısı eklendi. Dış katmanın sıralaması birim testle (`mob_model`) kapatıldı.

## Sapmalar (açık)

- Ses (iniş, sıçrayış, ezilme), iniş parçacıkları ve sudaki sıçrama etkisi yok.
- Görünmez (`invisible`) mob'lar için hiçbir mob'ta gövde gizlenmiyor; slime'ın dış katmanı da.
- Dış katman, öteki mob'larla aynı vertex listesinde ve aynı blend hattıyla çizilir; çok slime'ın
  birbirine göre karışım sırası kamera uzaklığına göre sıralanmaz (yalnız bir modelin dörtgenleri).
- Kutu yalnız boyutla ölçeklenir; `scale` özniteliği kutuya taşınmıyor (öteki mob'larla aynı).
- Sulfur cube (aynı taban sınıf) T37b'dedir.
