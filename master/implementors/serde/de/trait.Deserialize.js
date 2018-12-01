(function() {var implementors = {};
implementors["extprim"] = [{text:"impl&lt;'de&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"extprim/u128/struct.u128.html\" title=\"struct extprim::u128::u128\">u128</a>",synthetic:false,types:["extprim::u128::u128"]},{text:"impl&lt;'de&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"extprim/i128/struct.i128.html\" title=\"struct extprim::i128::i128\">i128</a>",synthetic:false,types:["extprim::i128::i128"]},];
implementors["serde_bytes"] = [{text:"impl&lt;'a, 'de: 'a&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"serde_bytes/struct.Bytes.html\" title=\"struct serde_bytes::Bytes\">Bytes</a>&lt;'a&gt;",synthetic:false,types:["serde_bytes::Bytes"]},{text:"impl&lt;'de&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"serde_bytes/struct.ByteBuf.html\" title=\"struct serde_bytes::ByteBuf\">ByteBuf</a>",synthetic:false,types:["serde_bytes::bytebuf::ByteBuf"]},];
implementors["serde_mtproto"] = [{text:"impl&lt;'de, T&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"serde_mtproto/wrappers/struct.Boxed.html\" title=\"struct serde_mtproto::wrappers::Boxed\">Boxed</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; + <a class=\"trait\" href=\"serde_mtproto/trait.Identifiable.html\" title=\"trait serde_mtproto::Identifiable\">Identifiable</a>,&nbsp;</span>",synthetic:false,types:["serde_mtproto::wrappers::Boxed"]},{text:"impl&lt;'de, T&gt; <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"serde_mtproto/wrappers/struct.WithSize.html\" title=\"struct serde_mtproto::wrappers::WithSize\">WithSize</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; + <a class=\"trait\" href=\"serde_mtproto/trait.MtProtoSized.html\" title=\"trait serde_mtproto::MtProtoSized\">MtProtoSized</a>,&nbsp;</span>",synthetic:false,types:["serde_mtproto::wrappers::WithSize"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()