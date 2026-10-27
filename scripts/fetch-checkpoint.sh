#!/usr/bin/env bash
# Downloads what `spark-x2-6 compile` and the reference read into $1, each file pinned by revision
# and checked by sha256 (files already present are only checked): XHToken's Spark-X2.6-1.7B
# weights, its tokenizer and chat template, its configuration and modeling code, and its license.
set -euo pipefail
DIR="$1"
MODEL="https://huggingface.co/XHToken/Spark-X2.6-1.7B/resolve/f18c032c7675f807c0fdffe281dbe124edba2605"
FILES="
525c465ab952779d2d360a9ff36f7f49ae018a23d80302ff839a850a77784c36  LICENSE
480576bd2ef80726a0dba6406d72ca37f4f4f9ca747c7f1c8d159dbdf40f6929  chat_template.jinja
16aa5d3619d0d893e7157f2499675430844eb7b383c80ecc1b3e8e8928a75d52  config.json
02218597240490f490659b184052db940b08163ff9c3af7b7e59323f6f963722  configuration_spark.py
804a5276a9d237600688267dd2bf6f9d4129ec21faf417f870d5d52b32ad9e6c  generation_config.json
b06fc88940ae6a20f71142f06304c12547f58ab52c175e56af12a38c3dc62f36  merges.txt
9cf0d1ad2b54b9f7088792779ddd8b5cb4d4fe63bfea054da7f2dd16362bcaf4  modeling_spark.py
48a257e3f5f20a004e7c30482a70ae19ae7d9cf713144f2a1394ba20af8c3cb5  special_tokens_map.json
710cce15cf3565674c499f9413997c6e8101f2bdd96245cff8f0311fb501248c  tokenizer.json
57b385dcc2ad031aef4a3786e97cd9a832ecb839cb27ed2cd829d627c90e1bfd  tokenizer_config.json
9f42cb7e7af5afe27cae2337d272d60b5659e10143ef4bb952f642b847916fe1  vocab.json
cc2b212985f5d0469bf926903e4bd7ee81856687baa0c6c6b55ff200d4cdc63f  model.safetensors.index.json
4461aaca4cac9b394ed951697e95a6da3c8952ed5976bbc162c5b085bdc0924a  model-00001-of-00002.safetensors
aea734125f5ac9c460847b33836977fe6afa68861b2d687f82bfe7d2b544d699  model-00002-of-00002.safetensors
"
mkdir -p "$DIR"
while read -r _ file; do
  if [ -z "$file" ] || [ -s "$DIR/$file" ]; then continue; fi
  echo "downloading $file"
  curl -fsSL --retry 3 -o "$DIR/$file.part" "$MODEL/$file"
  mv "$DIR/$file.part" "$DIR/$file"
done <<< "$FILES"
(cd "$DIR" && awk 'NF { print $1 "  " $2 }' <<< "$FILES" | shasum -a 256 -c --quiet -)
