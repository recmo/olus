


```sh
tree-sitter generate && tree-sitter parse ../../examples/test.olus
```


```sh
~/Library/Application\ Support/Zed/extensions/build/wasi-sdk/bin/clang -fPIC -shared -Os -Wl,--export=tree_sitter_olus -o olus.wasm -I ./src ./src/parser.c
```
