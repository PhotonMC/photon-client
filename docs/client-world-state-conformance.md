# P08.a/b — [server-conformance] ClientWorld chunk kopyası ve section cache

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Bu kayıt, vanilla sunucusundan alınan chunk section verisinin Rust tarafında
sorgulanabilir dünya durumuna yazıldığını gösterir. Kanıt sınıfı
`server-conformance`'tır: Rust bağlantısı vanilla sunucusundan gelen paketi
kabul edip `ClientWorld` içine işler. Java istemcisiyle aynı girdinin state
karşılaştırması yapılmadığından bu kayıt `client-parity` değildir ve P08'i
kapatmaz.

## Sözleşme

- `ClientWorld`, dimension sınırlarını ve `(chunk_x, chunk_z)` anahtarlı chunk
  deposunu taşır. Pinned vanilla overworld için varsayılan sınır `min_y=-64`,
  `height=384` ve dolayısıyla 24 section'dır.
- Chunk packet'inin section sırası dimension'ın minimum section Y değerinden
  başlar. Her section 16×16×16 block-state hücresini ve 4×4×4 biome hücresini
  saklar.
- Palette storage'ın bütün hücreleri Java'nın düşük bitten başlayan düzeniyle
  global ID'ye çözülür. Block index'i `y * 256 + z * 16 + x`, biome index'i
  `(y / 4) * 16 + (z / 4) * 4 + (x / 4)` biçimindedir.
- Dünya koordinatları chunk ve section'a floor/Euclidean bölme ile ayrılır;
  bu nedenle `-1` koordinatı `-1` chunk'ının yerel `15` hücresidir.
- Aynı chunk yeniden geldiğinde section cache'i eski column'ın yerine geçer.
  `unload_chunk` column'ı ve section'larını kaldırır. `tick` dünya saatini
  ilerletir; bu dilimde henüz entity veya ışık güncellemesi çalıştırmaz.
- Her parse edilen `ClientboundLevelChunkWithLightPacket`, yalnızca sayaçlara
  yazılmak yerine `PlayConnectionTrace.world` içindeki `ClientWorld`'e uygulanır.

`ClientWorld`'ün varsayılan dimension sınırı şu an login/respawn paketinden
dinamik olarak okunmaz; bu kayıt vanilla overworld test koşusuyla sınırlıdır.
Dimension registry çözümü ve dimension/respawn geçişi P08.a'nın açık işidir.

## Rust doğrulaması

Yerel davranış testleri tam palette çözümünü, 4096/64 hücre uzunluklarını,
negatif chunk koordinatlarını, section metadata'sını, replace/unload cache
semantiğini ve bağımsız tick sayacını kontrol eder:

```text
cargo test --offline
52 passed; 0 failed
```

Parser'ın canlı vanilla koşusunda kullandığı ölçüm:

```text
cargo run --offline --release --example connection_probe -- 127.0.0.1:25566 WorldStateC26
```

İlgili çıktı:

```text
connection=ok
play_login=true
player_loaded_sent=true
chunk_packet_count=24
decoded_chunk_count=24
world_dimension=min_y:-64,height:384
world_chunk_count=24
world_section_count=576
world_game_time=0
first_world_state=chunk_x:0,chunk_z:1,section_y:-4,block_state_at_origin:Some(85),biome_at_origin:Some(21),section_counts:Some((4096, 0))
```

24 alınan chunk'ın 24 section'ı dünya cache'inde 576 section olarak tutuldu;
ilk chunk'ın origin sorgusu palette'den çözülmüş block-state `85` ve biome `21`
döndürdü. Sunucu oturumunda protocol error oluşmadı.

## Açık kapsam

Bu kayıt Java istemci state'iyle parity ölçmez. Dimension registry'den min Y ve
yükseklik okuma, respawn ile world değiştirme, ışık nibble değerleri, block
entity world state'i, block update ile section değiştirme, radius/unload sırası,
entity deposu ve render için section invalidation P08'in kalan işleridir.
Chunk packet'inin wire alanları ve ışık byte sınırlarına ait önceki kayıt
[chunk paketinin yapısal doğrulamasında](chunk-packet-conformance.md) bulunur.
