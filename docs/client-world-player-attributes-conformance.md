# P08.j — [server-conformance] LocalPlayer attribute state'i ve `dataToKeep` bit 1

Bu kayıt, vanilla attribute update packet'inin local player state'ine
uygulanmasını ve respawn sırasında `dataToKeep` bit 1'in attribute
modifier'larına etkisini kapsar. Canlı kanıt sınıfı `server-conformance`'dır:
Rust probe'u Mojang vanilla 26.2 dedicated server'ına bağlanır, vanilla'nın
initial attribute snapshot'ını alır, Nether respawn'ını tetikler ve aynı
attribute state'inin yeni local player nesline taşındığını sorgular. Aynı girdi
Java istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundUpdateAttributesPacket.java:19-80`, packet entity ID'sini VarInt
olarak ve en fazla `128` attribute snapshot'ını listeler. Her snapshot
attribute registry holder ID'sini, base `double` değerini ve modifier
collection'ını taşır. Modifier sırası identifier, amount `double` ve operation
ID'sidir. `Attribute.java:12-15` holder'ın registry üzerinden codec edildiğini;
`AttributeModifier.java:14-64` operation ID'lerinin `0=add_value`,
`1=add_multiplied_base`, `2=add_multiplied_total` olduğunu gösterir.

`ClientPacketListener.java:2348-2372`, entity mevcutsa her snapshot için
attribute instance'ının base değerini değiştirir, mevcut modifier'ları siler ve
packet'teki modifier'ları transient olarak ekler. Bilinmeyen entity sessizce
atlanır; living olmayan bilinen entity attribute packet'i alırsa Java hata
üretir. `ServerEntity.java:286-290,349-356`, syncable attribute'ların entity
eklenirken ve dirty olduklarında aynı packet türüyle gönderildiğini gösterir.

`AttributeMap.java:92-107`, `assignAllValues` yolunun base değer ve modifier'ı,
`assignBaseValues` yolunun yalnız base değerleri yeni attribute map'ine
aktardığını gösterir. Respawn'daki bit 1 bu iki yol arasındaki seçimi yapar.
Packet dispatch kaydı `GameProtocols.java:264` ve
`GamePacketTypes.java:130`'dadır; protokol 776'da clientbound update-attributes
ID'si `131`'dir.

## Sözleşme

- Update-attributes packet entity ID'si ve snapshot listesi packet sonuna kadar
  tüketilir; attribute registry ID'si ve modifier operation ID'si ham numeric
  değer olarak korunur.
- Snapshot'taki her attribute için base değeri ve modifier listesi birlikte
  saklanır. Aynı attribute tekrar gelirse önceki snapshot'ın yerine geçer;
  packet'te bulunmayan mevcut attribute state'i korunur.
- Packet entity ID'si local player ID'siyle eşleştiğinde snapshot'lar local
  player world state'ine uygulanır. Diğer entity ID'leri packet gözleminde
  tutulur, local player state'ini değiştirmez.
- Respawn yeni local player nesli oluştururken attribute base değerlerini her
  iki maskede taşır. `dataToKeep` bit 1 açıksa modifier'lar da taşınır; kapalıysa
  modifier listeleri temizlenir.
- Bu dilim effective attribute hesabını, modifier operasyonlarının hareket/
  health/combat etkilerini ve attribute registry ID'sinin isim çözümlemesini
  kapsamaz.

## Rust doğrulaması

`parse_update_attributes_packet`, Java stream codec alan sırasını ve packet
sonunu doğrular. `ClientAttributeState` base değerini, registry ID'sini ve
modifier listesini; `ClientAttributeModifierState` identifier, amount ve
operation ID'sini sorgulanabilir state olarak tutar.

`ClientWorld::apply_local_player_attributes`, yalnız local player ID'siyle
eşleşen packet'i state'e yazar. `ClientWorld::recreate_local_player`, Java'nın
`assignAllValues`/`assignBaseValues` ayrımını `dataToKeep` bit 1 ile uygular.
Respawn observation'ı yeniden kurulan local player'daki attribute ve modifier
adetlerini ayrıca kaydeder.

Unit testler packet field order'ını, raw double/identifier/operation değerlerini,
trailing byte reddini ve bit 1 açık/kapalı respawn sonuçlarını doğrular. Probe
örneği initial local player snapshot'ının base/modifier adetleriyle respawn
sonrası world state'ini karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe çalışırken vanilla server konsolunda her oturum için şu komut çalıştırıldı:

```text
execute as <probe-user> in minecraft:the_nether run tp @s 0 80 0
```

Rust probe:

```text
cargo run --locked --offline --example world_player_respawn_probe -- 127.0.0.1:25568 <probe-user>
```

İki tamamlanmış vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08jA2` | `P08jB` |
| --- | ---: | ---: |
| login player ID | `3165` | `2966` |
| total attribute update observations | `125` | `64` |
| local player attribute update count | `1` | `1` |
| initial local player attribute count | `3` | `3` |
| initial local player modifier count | `2` | `2` |
| respawn packet count | `1` | `1` |
| respawn dimension | `minecraft:the_nether` | `minecraft:the_nether` |
| `data_to_keep` | `3` | `3` |
| `keep_attribute_modifiers` | `true` | `true` |
| respawn local player attribute count | `3` | `3` |
| respawn local player modifier count | `2` | `2` |
| local player ID after respawn | `3165` | `2966` |
| local player generation | `1` | `1` |
| final local player attribute count | `3` | `3` |
| final local player modifier count | `2` | `2` |
| dimension changed | `true` | `true` |
| final world chunk count | `157` | `157` |

Her iki oturumda vanilla'nın local player için tek attribute update snapshot'ı
3 attribute ve toplam 2 modifier içerdi. Respawn packet'i `dataToKeep=3` ile
geldi; Rust yeni nesli aynı ID ile oluşturdu ve 3 attribute/2 modifier state'ini
korudu. Toplam packet gözlem sayısının entity yayın sırasına göre değişmesi
başarı ölçütü yapılmadı.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve `ClientWorld` attribute
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Effective attribute değerinin modifier operasyonlarıyla hesaplanması, attribute
registry isimleri, movement/health/combat etkileri, remote living entity
attribute state'inin world map'inde tutulması, equipment ve menu/inventory/effect
state'i, datapack custom dimension ve `client-parity` karşılaştırması açık kalır.
