# Mth getInt davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `getInt(String, int)` yordamını ve onun pinned Apache Commons Lang 3.20.0 bağımlılığını kapsar.

`getInt`, `NumberUtils.toInt` üzerinden `Integer.parseInt` çağrısını kullanır. Geçerli decimal metin, isteğe bağlı ASCII `+` veya `-` işareti ve Java `Character.digit` tarafından tanınan decimal karakterlerle parse edilir. Boşluklar otomatik temizlenmez. Null, boş metin, tek başına işaret, biçim hatası ve i32 aralığı dışındaki sonuçlarda yardımcı default değeri döndürür.

Behavior runner `input` alanını string veya JSON null, `default` alanını signed decimal i32 string olarak taşır. Java oracle pinned client JAR ve commons-lang3 3.20.0 ile doğrudan `Mth.getInt` çağırır; beklenen sonuçlar Rust uygulamasından türetilmez. Rust tarafı Java'nın decimal digit ve i32 sınırlarını aynı parse sırasıyla uygular.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:279-281`; bağımlılık kimliği `org.apache.commons:commons-lang3:3.20.0`.

Kapsam dışı kalanlar: `smallestEncompassingPowerOfTwo`, `frac`, `getSeed`, JSON sayısal türlerinin metne dönüştürülmesi ve `getInt` kullanan istemci çağrı zincirleri.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `mth_get_int`, Java `Mth.getInt(String, int)` yolunu ve pinned Apache Commons Lang `NumberUtils.toInt` fallback davranışını JSONL adapter'a bağlar. Decimal parse, isteğe bağlı ASCII işaret, Java decimal digit kuralları, null/default, biçim hatası ve i32 sınırları Java oracle'ın signed decimal sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Null/default, ASCII işaret, min/max sınırları, taşma, boşluk, biçim hataları ve BMP Unicode decimal digit yollarını içeren 29 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ac76a3ceda5d6328e658a3fb88d207e8283bde484c2017637a1439510e0b0f43`. Birleşik referans kimliği: `3c7458b1449a1f03f8ee740e863fddf577010fa64c65fa62e8f0bdbcea4de854`.

Adapter hashleri: debug `c4b7b74f0992d10d4f801f9f649d2c78a456440021800201238f7d6e7a39e1ad`, release `83042bb80924dad2a4707e456a2f4a4076dbb6971aabd60b4fbeffe22d4cf7dc`.

Yerel raporlar: `compare-osgiuhm_` ve `compare-9wal3e07` (math debug/release), `compare-b89cg5u0` ve `compare-ip_sonvb` (birleşik debug/release), `verify-k3n6nlq9` (math) ve `verify-iciu4g12` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.getInt` yolu, commons-lang3 3.20.0 bağımlılığı ve bu dilimin 29 sentetik girdisiyle sınırlıdır. `smallestEncompassingPowerOfTwo`, `frac`, `getSeed`, JSON sayısal türlerinin metne dönüştürülmesi ve `getInt` kullanan istemci çağrı zincirleri ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
