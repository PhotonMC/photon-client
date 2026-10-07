# P04.c — Kaynak reload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

`ReloadableResourceManager` boş bir aktif kaynak manager'ı ile başlar. Her reload çağrısı eski aktif pack stack'ini kapatır ve yeni pack listesinden bir manager kurar; reload tamamlandıktan sonra sorgular yalnızca yeni listedeki kaynakları görür.

Yeni pack listesi verilen sırayı korur. Aynı identifier birden fazla yeni pack'te varsa son pack seçilir ve kaynak stack'i düşük öncelikliden yüksek öncelikliye döner. Aktif namespace'ler yalnızca yeni pack listesindeki kaynaklardan oluşur; adapter çıktısında karşılaştırılabilirlik için sıralanır.

Bu dilim listener hazırlama sırası, executor zamanlaması, reload hataları, pack metadata/filter/overlay ve repository keşfini kapsamaz. Test girdileri özgün sentetik pack kimlikleri ve byte dizileridir.


## Client-parity doğrulaması

Java oracle, sentetik `PathPackResources` listesini `ReloadableResourceManager.createReload` içine verir. Her turda reload tamamlandıktan sonra aktif pack kimlikleri, namespace'ler, seçilen kaynak/byte dizisi ve sorgu stack'i karşılaştırılır; ilk snapshot manager'ın boş başlangıcını gözlemler.

Kapsanan durumlar:

- boş manager başlangıcı ve boş pack reload'u;
- ortak kaynağın yeni stack içinde son pack'ten seçilmesi;
- önceki turdaki kaynağın sonraki reload'da görünmemesi;
- namespace ve aktif pack listesinin yeni stack'e göre değişmesi.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- sekiz özgün senaryo iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `c17defe6bb454349dc39aea4a9e49630e62ff6263caf7e936f9b7ffdc996ad2d`;
- Java `verify` raporu `verify-26o87t66`;
- Rust debug karşılaştırması `compare-r7zvo5bh`, release karşılaştırması `compare-62jmlzev` ve her ikisi de fark üretmedi.

Tam corpus capture kimliği `b31c28a680797feb6ed758ece706e3e81280da626edb851b6da5258fdcccc591`; debug/release karşılaştırmaları `compare-flms3_ct` ve `compare-78rawyo7`, Java doğrulaması `verify-qqkqtkvy` olarak kaydedildi.

Listener zamanlaması, metadata/filter/overlay ve repository keşfi sonraki P04 dilimleridir.
