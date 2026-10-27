# Spark-X2.6-1.7B in the browser. `just ui` builds everything, the model included, and serves the
# page; `just build` then `just serve` do the same in two steps.

import 'vendor/fusor-ai/just/product.just'

name := "spark-x2-6"
# The bundle is 1.8 GB, the conversation's state and the activations on top of it.
export MAX_MEMORY := "4294967296"
checkpoint := env_var_or_default("CHECKPOINT", ".cache/checkpoint")
conversations := "fixtures/conversations.json"
goldens := "fixtures/goldens"
# The reference the goldens come from (vendor/fusor-ai/models/spark-x2-6/reference): the
# checkpoint's own modeling code under transformers, pinned.
reference := "uv run --no-project --python 3.12 --with transformers==4.57.1 --with torch==2.9.1 --with numpy==2.5.4 --with safetensors==0.8.0 --with huggingface-hub==0.36.2 python"

host:
    bash -n scripts/fetch-checkpoint.sh
    node --check app/web/app.js
    node --check app/engine/worker.js
    node --check scripts/node-smoke.mjs
    node --check scripts/agreement.mjs
    node --check scripts/demo.mjs
    node --check scripts/render-images.mjs
    node --check scripts/site-bundle.mjs
    node --check e2e/run.mjs

# Downloads the pinned checkpoint (3.4 GB, sha256-checked) unless it is already there, then
# compiles the bundle into dist/model, its license beside it.
model: native
    scripts/fetch-checkpoint.sh {{checkpoint}}
    rm -rf {{model}}
    target/release/spark-x2-6 compile {{checkpoint}} {{model}}
    cp {{checkpoint}}/LICENSE {{model}}/

# Try it on localhost. The bundle and the runtimes are rebuilt every time (incremental, seconds
# when nothing changed), so they always come from the same fusor-ai.
ui: model runtimes page serve

# Every stage against the reference's goldens (vendor/fusor-ai/models/spark-x2-6/reference).
parity: model
    test -f {{goldens}}/golden_f32_colors.safetensors || just goldens
    target/release/spark-x2-6 parity {{model}} {{goldens}}/golden_f32_colors.safetensors {{goldens}}/golden_f32_trip.safetensors

# Golden tensors of every conversation, and the reference's replies (fixtures/references.json),
# from the reference in f32 on the CPU.
goldens:
    scripts/fetch-checkpoint.sh {{checkpoint}}
    {{reference}} {{fusor_ai}}/models/spark-x2-6/reference/dump.py {{checkpoint}} {{conversations}} {{goldens}} fixtures/references.json

# Every conversation of the fixtures, replied to by the native CLI.
chat: native
    target/release/spark-x2-6 chat {{model}} {{conversations}}

bench case="colors": native
    target/release/spark-x2-6 bench {{model}} {{conversations}} {{case}}

# The README's demo, assets/demo/demo.gif: the real page, served with dist/model, recorded in
# headless Chrome while it answers the first example. ffmpeg comes from imageio-ffmpeg.
demo: model runtimes page
    cd vendor/fusor-ai/packages/e2e && npm ci --no-audit --no-fund --silent && npx playwright install chromium
    FFMPEG=$(uv run --no-project --with imageio-ffmpeg==0.6.0 python -c 'import imageio_ffmpeg; print(imageio_ffmpeg.get_ffmpeg_exe())') node scripts/demo.mjs

# The README's banner and benchmark summary, light and dark.
images:
    cd vendor/fusor-ai/packages/e2e && npm ci --no-audit --no-fund --silent && npx playwright install chromium
    node scripts/render-images.mjs

# Its tensor parts are named by their BLAKE3, so /model/parts/ can be cached as immutable.
# The page, its four runtimes and the bundle beside it, as a static site in target/site.
site: runtimes page
    rm -rf target/site
    mkdir -p target/site
    cp -R {{app}}/dist/. target/site/
    find target/site -name '.fusor-*.json' -delete
    node scripts/site-bundle.mjs {{model}} target/site/model
    cp {{model}}/LICENSE target/site/model/

# The Space holds at most 1 GB, so it gets the page, the runtimes and the model's licenses, and the
# page loads the bundle from the model repo fusorlabs/spark-x2.6-1.7b-web (`just deploy-model`). The Space must
# already exist as a static Space: creating it with the upload makes a Gradio Space, which
# Hugging Face refuses for a free organization.
# The site to the Hugging Face Space fusorlabs/spark-x2-6-wasm, configured by space/README.md.
deploy-space: site
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf target/space
    rsync -a --exclude /model target/site/ target/space/
    rsync -a --include '*/' --include 'LICENSE*' --include NOTICE --exclude '*' target/site/model/ target/space/model/
    cp space/README.md target/space/
    uv run --no-project --with huggingface_hub==0.36.2 python - target/space <<'PY'
    import sys
    from huggingface_hub import HfApi

    HfApi().upload_folder(
        repo_id="fusorlabs/spark-x2-6-wasm",
        repo_type="space",
        folder_path=sys.argv[1],
        delete_patterns="*",
        commit_message="Deploy spark-x2-6-wasm {{version}}",
    )
    PY

# Deploy it before the Space whenever the bundle changes: the runtime only loads a bundle compiled
# with the same fusor-ai.
# The site's bundle, with space/MODEL.md as its card, to the model repo fusorlabs/spark-x2.6-1.7b-web.
deploy-model: site
    #!/usr/bin/env bash
    set -euo pipefail
    cp space/MODEL.md target/site/model/README.md
    uv run --no-project --with huggingface_hub==0.36.2 python - target/site/model <<'PY'
    import sys
    from huggingface_hub import HfApi

    api = HfApi()
    api.create_repo("fusorlabs/spark-x2.6-1.7b-web", exist_ok=True)
    api.upload_folder(
        repo_id="fusorlabs/spark-x2.6-1.7b-web",
        folder_path=sys.argv[1],
        delete_patterns="*",
        commit_message="Deploy the spark-x2-6-wasm {{version}} bundle",
    )
    PY

# The fixed inputs the checks read.
fixtures:
