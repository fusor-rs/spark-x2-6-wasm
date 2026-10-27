---
license: apache-2.0
base_model:
  - XHToken/Spark-X2.6-1.7B
pipeline_tag: text-generation
library_name: fusor-ai
tags:
  - webassembly
  - wasm
  - on-device
---

# Spark-X2.6-1.7B, compiled for the browser

XHToken's [Spark-X2.6-1.7B](https://huggingface.co/XHToken/Spark-X2.6-1.7B) compiled into a
[fusor-ai](https://github.com/fusor-rs/fusor-ai) bundle: the weights the browser runtime of
[spark-x2-6-wasm](https://github.com/fusor-rs/spark-x2-6-wasm) loads to hold a text conversation
on the device's CPU. Try it in the Space
[fusorlabs/spark-x2-6-wasm](https://huggingface.co/spaces/fusorlabs/spark-x2-6-wasm).

- **Source:** `XHToken/Spark-X2.6-1.7B` at revision `f18c032c7675f807c0fdffe281dbe124edba2605`,
  compiled with the `q7` profile: every large matrix 7-bit (1921 MB).
- **Format:** `manifest.json` lists every tensor and part; each part is named by its BLAKE3 hash,
  which the runtime checks as it loads.
- **Reproduce it:** `just model` in spark-x2-6-wasm downloads the checkpoint and compiles the same
  bundle.

This is not a PyTorch or safetensors checkpoint: it only loads in the spark-x2-6-wasm runtime.

The model is Apache-2.0 ([LICENSE](LICENSE)).
