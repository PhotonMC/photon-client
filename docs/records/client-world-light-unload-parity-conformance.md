# P08.c-light.2 — ClientWorld light unload cleanup — [client-parity]

Bu kayıt, Java istemcisinin forget-chunk yolunda queued ve görünür sky/block
light section verisini temizlemesini kapsar. Kanıt sınıfı `client-parity`'dir:
aynı özgün trace Java 26.2 `LevelLightEngine` fixture'ına ve production Rust
`ClientWorld` adapter'ına verilir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `client/multiplayer/ClientPacketListener.java:909-932`,
  `handleForgetLevelChunk` chunk kaynağını düşürür ve `queueLightRemoval`
  runnable'ını level light queue'ya bırakır. Runnable light column'ını kapatır,
  light section aralığındaki sky/block queued verisini `null` ile siler ve
  dimension section'larını empty olarak işaretler.
- `client/multiplayer/ClientLevel.java:493-505`, light queue çalıştırıldıktan
  sonra `LevelLightEngine.runLightUpdates` çağrısını yapar; `unload` ayrıca
  chunk'ın light column'ını kapatır.
- `world/level/lighting/LayerLightSectionStorage.java:203-214,227-253`,
  `null` section verisinin queued kaydı kaldırdığını, empty/non-empty section
  durumunun layer oluşturma veya kaldırma kararını verdiğini gösterir.

Fixture, bu gerçek engine metotlarını doğrudan çağırır. Chunk getter'ı
propagation üretmediği için trace yalnız forget temizliği, section status ve
görünür data sorgusunu ölçer.

## Yeni dal ve varyant kararı

Bu dilim, önceki [P08.c-light.1 queue/publication kaydından](client-world-light-queue-parity-conformance.md)
ayrı olarak Java `handleForgetLevelChunk` içindeki `queueLightRemoval`
dalını ilk kez çalıştırır. Farklı gözlem, unload sırasında hem görünür light
map'inin hem de yüklenmemiş chunk için bekleyen layer'ların silinmesi ve aynı
chunk yeniden yüklendiğinde eski verinin geri dönmemesidir. Aynı sonucu veren
mevcut `client-parity` kayıtlarının sayısı **0**'dır; önceki unload kaydı
`server-conformance` olduğu için bu sayıya dahil değildir.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Dimension bounds
  `min_y=-64`, `height=384` olarak sabitlenir; level section aralığı `-4..19`,
  light section aralığı `-5..20`'dir.
- Yüklü olmayan chunk'a gelen data layer önce queued state'te tutulur.
  Forget işlemi light section aralığındaki sky ve block queued kayıtlarını
  kaldırır; bu kayıtlar daha sonra chunk yüklenince yayınlanmaz.
- Forget işlemi yüklü chunk'ın tüm section ve light state'ini kaldırır.
  Aynı chunk non-air section ile yeniden kurulduğunda Java'nın section status
  yolu iki layer için sıfır `DataLayer` oluşturur; önceki light değeri geri
  gelmez.
- Yüklü non-air section için sky ve block sorguları, açık bir paket layer'ı
  gelene kadar `0` döndürür. Air section için data layer oluşmadığında sonuç
  yoktur; unload edilmiş chunk için sonuç `None`/null'dır.

Trace, `0x0F` ve `0xA6` değerlerini unload öncesi queued state'e verir, forget
sonrası sorgular, chunk'ı non-air section ile yeniden yükleyip sıfır sonuçlarını
ölçer, ardından `0x26` ve `0x37` layer'larını uygular ve ikinci forget sonrası
değerlerin yok olduğunu sorgular.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/light-unload-removal` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`46b6963e24864014d6b32a3245d9913023c7723b6cf42720d15878263c5bd8c3`

Math corpus'u **2969** kayıttır; fark `null` kalmıştır. Yeni kaydın
gözlenebilir sonuçları queued ve loaded forget sonrasında light sorgularının
boş olması, yeniden yüklenen non-air section'da sky/block değerlerinin `0`
olması ve güncel layer'da sırasıyla `6/7` görülmesidir.

## Rust doğrulaması

`ClientWorld::replace_chunk`, Java section status yolunun gözlenebilir kısmını
korumak için non-air section'lara sıfır sky/block light layer'ı bağlar; air
section'lar implicit light data almaz. `ClientWorld::unload_chunk` chunk ile
birlikte loaded light map'ini ve o chunk'a ait pending light update'larını
atar. Adapter, Java Gson çıktısındaki null alanların serileştirme biçimini
koruyarak yalnız mevcut light değeri için `value` alanı üretir.

Debug adapter SHA-256 değeri
`8da9cf3007eb76c95e9474691e0d5f9da3d3aead95e59475effb713dd397cc4c` ile aynı
capture üzerinde **2969/2969** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`e5c8ed65d94c039725398ce89157ee4b04a1b8b1f67ddd868b07a4fa16d384a7` ile aynı
capture üzerinde **2969/2969** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2969
difference=null
status=parity_passed

cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2969
difference=null
status=parity_passed
```

## Açık kapsam

Bu kayıt light propagation/relight hesaplarını, komşu section dirty/rebuild
bildirimlerini, gerçek `ClientLevel` frame queue'sunu, render/AO invalidation'ı,
sky default brightness'ı ve dimension/respawn ışık yaşam döngüsünü kapsamaz.
Vanilla sunucusuna canlı bağlantı, bu kaydın client-parity kanıtının yerine
geçmez.
