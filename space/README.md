---
title: spark-x2-6-wasm
short_description: Chat with Spark-X2.6-1.7B, thinking first, on your CPU
sdk: static
app_file: index.html
pinned: false
license: apache-2.0
models:
  - fusorlabs/spark-x2.6-1.7b-web
  - XHToken/Spark-X2.6-1.7B
custom_headers:
  cross-origin-embedder-policy: require-corp
  cross-origin-opener-policy: same-origin
  cross-origin-resource-policy: cross-origin
---

A text conversation with XHToken's
[Spark-X2.6-1.7B](https://huggingface.co/XHToken/Spark-X2.6-1.7B), computed on your own CPU in
the browser: the model, compiled to WebAssembly with fusor-ai, downloads once and then runs sealed
from the network, reasoning first when asked and writing each reply as you read it. The model is
Apache-2.0; its license ships beside the bundle as `model/LICENSE`. The page's code is MIT; its
source is [github.com/fusor-rs/spark-x2-6-wasm](https://github.com/fusor-rs/spark-x2-6-wasm).
