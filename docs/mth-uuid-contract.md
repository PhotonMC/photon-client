# Mth createInsecureUUID davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `createInsecureUUID(RandomSource)` yordamını kapsar. On iki sabit seed ile iki `nextLong` çekişi ve UUID bit maskeleri sınanır.

Yordam ilk çekişteki long'un version bitlerini `& -61441L` ile temizleyip `| 16384L` ile version 4'ü kurar. İkinci çekişteki long'un variant bitlerini `& 4611686018427387903L` ile temizleyip `| Long.MIN_VALUE` ile RFC 4122 varyantını kurar. Sonuç iki 64-bit yarı olarak taşınır; UUID string biçimi ve byte codec'i bu dilimin dışındadır.

Behavior runner seed'i signed decimal i64 string olarak alır; `most_hex` ve `least_hex` sonuçları 16 haneli lowercase unsigned hexadecimal string'dir. Java oracle `Mth.createInsecureUUID` yordamını doğrudan çağırır; beklenen yarılar Rust üretiminden türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:338-342`.

Kapsam dışı kalanlar: `Mth.getSeed`, `RandomSource` dizisinin kendisi, UUID byte okuma/yazma ve istemcinin UUID tüketim sıraları.
