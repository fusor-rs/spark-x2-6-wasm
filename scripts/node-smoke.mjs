// Runs a single-threaded wasm runtime build in Node against the bundle: pushes every part the way
// the browser loader does, then holds each conversation of fixtures/conversations.json turn by
// turn, step by step as the page does, and prints each reply's speed, the facts it states and how
// much of the reference's reply (fixtures/references.json) it repeats. With `--out`, each
// conversation's replies are written to `<dir>/<conversation>.txt`.
//
//   node scripts/node-smoke.mjs dist/model --build web-single [--out dir]

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { parseArgs } from 'node:util';
import { agreement, missing, shown } from './agreement.mjs';

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    build: { type: 'string', default: 'web-single' },
    out: { type: 'string' },
  },
});
const CHUNK = 8 << 20;

const bundle = positionals[0] ?? 'dist/model';
const dist = new URL(`../app/engine/dist/${values.build}/`, import.meta.url);
const glue = await import(new URL('spark_x2_6_runtime.js', dist));
glue.initSync({ module: readFileSync(new URL('spark_x2_6_runtime_bg.wasm', dist)) });
const conversations = JSON.parse(readFileSync('fixtures/conversations.json', 'utf8'));
const references = JSON.parse(readFileSync('fixtures/references.json', 'utf8'));
if (values.out) mkdirSync(values.out, { recursive: true });
const runtime = load(bundle);
let failed = false;
for (const [name, { turns }] of Object.entries(conversations)) {
  runtime.reset();
  const replies = [];
  turns.forEach(({ expect = [], ...message }, index) => {
    const start = performance.now();
    const plan = JSON.parse(runtime.start(JSON.stringify(message)));
    let progress = JSON.parse(runtime.step());
    let first = null;
    while (!progress.done) {
      first ??= progress.tokens ? performance.now() : null;
      progress = JSON.parse(runtime.step());
    }
    const seconds = (performance.now() - start) / 1000;
    const written = (performance.now() - (first ?? start)) / 1000;
    const reply = progress.reasoning === null ? progress.answer : `${progress.reasoning}\n${progress.answer}`;
    const left = missing(expect, progress.answer);
    const repeated = agreement(shown(references[name][index], message.thinking), reply);
    console.log(`  ${name} ${index}: ${plan.prompt_tokens} tokens read, ${progress.tokens} written in ${seconds.toFixed(2)}s (${((progress.tokens - 1) / written).toFixed(1)} tokens/s); ${left.length ? `missing ${left.join(', ')}` : `states ${expect.join(', ') || 'nothing asked'}`}; ${(repeated * 100).toFixed(0)}% of the reference's reply repeated`);
    console.log(`    ${JSON.stringify(progress.answer)}`);
    failed ||= left.length > 0;
    replies.push(reply);
  });
  if (values.out) writeFileSync(join(values.out, `${name}.txt`), `${replies.join('\n\n---\n\n')}\n`);
}
runtime.free();
if (failed) process.exit(1);

/** A runtime with every part of the bundle in `bundle` pushed into it. */
function load(bundle) {
  const t0 = performance.now();
  const manifest = readFileSync(join(bundle, 'manifest.json'), 'utf8');
  const runtime = new glue.Runtime(manifest);
  let ready = false;
  JSON.parse(manifest).parts.forEach((part, index) => {
    const bytes = readFileSync(join(bundle, part.file));
    for (let o = 0; o < bytes.length; o += CHUNK) runtime.write_part(index, bytes.subarray(o, o + CHUNK));
    ready = runtime.end_part(index);
  });
  if (!ready) throw new Error(`${bundle}: engine not ready after every part`);
  console.log(`${values.build} ${bundle}: load ${((performance.now() - t0) / 1000).toFixed(2)}s`);
  return runtime;
}
