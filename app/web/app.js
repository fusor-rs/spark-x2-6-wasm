import { ComputePool, SPAWN_COMPUTE } from '../../vendor/fusor-ai/packages/host/compute.js';

// The engine bridge. The page's state lives in Rust (src/app.rs); this module owns the engine
// worker and, in threaded builds, its compute workers (fusor-ai's ComputePool), hands it the
// message `src/app.rs` composed when Send is pressed, measures each reply and reports back with
// `engine` events.

const params = new URLSearchParams(location.search);
const cores = navigator.hardwareConcurrency || 4;
// One compute worker per core minus two (the page and the engine worker), at most 8; `?threads=1`
// forces the single-threaded build. Never more than the cores: the barriers spin.
const requestedThreads = Number.parseInt(params.get('threads') ?? '', 10);
const THREADS = Math.min(cores, requestedThreads > 0 ? requestedThreads : Math.max(1, Math.min(8, cores - 2)));
// The bundle served next to the page (`just serve` mounts dist/model there); `?model=<url>` loads
// another one, `?build=web-single` pins a runtime build.
// A Hugging Face Space holds at most 1 GB, so the Space loads the bundle from the model repo
// fusorlabs/spark-x2.6-1.7b-web (`just deploy-model`) instead of from beside the page.
const HOSTED_MODEL = 'https://huggingface.co/fusorlabs/spark-x2.6-1.7b-web/resolve/main/';
const DEFAULT_MODEL = location.hostname.endsWith('.hf.space') ? HOSTED_MODEL : 'model/';
const MODEL_URL = new URL(params.get('model') ?? DEFAULT_MODEL, document.baseURI).href;

const mb = (bytes) => `${(bytes / 1e6).toFixed(1)} MB`;

export function onMount({ root, inputs, onCleanup }) {
  const report = (detail) => root.dispatchEvent(new CustomEvent('engine', { detail: JSON.stringify(detail) }));
  const engine = new Worker(new URL('engine/worker.js', document.baseURI), { type: 'module', name: 'spark-x2-6-engine' });
  const computePool = new ComputePool(crash);
  let build = null;
  let notice = '';
  let loaded = false;
  let broken = false;
  let jobs = 0;
  let job = null;
  let readyAt = Infinity;

  const handlers = {
    [SPAWN_COMPUTE]: (message) => computePool.spawn(message),
    runtime: (message) => (build = message),
    warning: ({ message }) => {
      console.warn(message);
      notice = ` (${message})`;
    },
    progress: ({ done, total, network, cache }) => {
      const source = cache && !network ? 'from this device' : cache ? `${mb(network)} downloaded, ${mb(cache)} cached` : 'downloading';
      report({ type: 'loading', fraction: done / total, detail: `${mb(done)} / ${mb(total)} · ${source}` });
    },
    ready: ({ ms, cache, total }) => {
      loaded = true;
      readyAt = performance.now();
      const loadedInfo = { build: `${build.build}${notice}`, threads: build.threads, bundle_bytes: total, load_ms: ms, from_device: cache === total };
      report({ type: 'ready', loaded: loadedInfo });
    },
    started: (message) => {
      if (message.job !== job?.id) return;
      job.plan = message.plan;
      report({ type: 'reply', status: 'Reading the conversation…', reasoning: null, answer: '', metrics: metrics() });
    },
    reply: (message) => {
      if (message.job !== job?.id) return;
      reply(message, 'Writing…');
    },
    done: (message) => {
      if (message.job !== job?.id) return;
      if (!message.stopped) reply(message, 'Done.');
      report({ type: 'done', status: message.stopped ? 'Stopped.' : 'Done.', metrics: metrics(message) });
      job = null;
    },
    error: ({ message }) => {
      if (!loaded) return fail(message);
      job = null;
      report({ type: 'error', message });
    },
    fatal: ({ message }) => fail(message),
  };

  engine.onmessage = ({ data }) => {
    if (!broken) handlers[data.type]?.(data);
  };
  engine.onerror = (event) => crash(event.message || 'the engine worker failed');
  engine.postMessage({ type: 'load', modelUrl: MODEL_URL, threads: THREADS, build: params.get('build') });

  /** The reply so far, as the step left it. */
  function reply(message, status) {
    const now = performance.now();
    if (message.tokens > 0) job.firstAt ??= now;
    job.lastAt = now;
    job.tokens = message.tokens;
    report({ type: 'reply', status, reasoning: message.reasoning, answer: message.answer, metrics: metrics(message) });
  }

  /** Sends the message `src/app.rs` composed, with a seed of its own. */
  function send() {
    job = { id: ++jobs, t0: performance.now(), firstAt: null, lastAt: null, plan: null, tokens: 0 };
    const [seed] = crypto.getRandomValues(new Uint32Array(1));
    const message = JSON.stringify({ ...JSON.parse(inputs.message.get()), seed });
    engine.postMessage({ type: 'send', job: job.id, message });
  }

  function stop() {
    if (job) engine.postMessage({ type: 'stop', job: job.id });
  }

  /** A new conversation: the engine forgets the last. */
  function reset() {
    engine.postMessage({ type: 'reset' });
  }

  function metrics(message = {}) {
    const ownRequests = performance.getEntriesByType('resource').filter((entry) => entry.startTime >= readyAt).length;
    const tokens = job?.tokens ?? 0;
    const promptTokens = job?.plan?.prompt_tokens ?? 0;
    return {
      first_ms: job?.firstAt ? job.firstAt - job.t0 : null,
      tokens,
      writing_ms: job?.firstAt ? job.lastAt - job.firstAt : 0,
      context_tokens: promptTokens + tokens,
      prompt_tokens: promptTokens,
      requests: ownRequests + (message.networkRequests ?? 0),
    };
  }

  /** The model could not load, or the engine or a compute thread died. */
  function fail(message) {
    broken = true;
    engine.terminate();
    computePool.terminate();
    report({ type: 'broken', message });
  }

  function crash(message) {
    fail(`The engine stopped: ${message}.`);
  }

  onCleanup(onClicks(inputs.sends, send));
  onCleanup(onClicks(inputs.stops, stop));
  onCleanup(onClicks(inputs.resets, reset));
  onCleanup(() => {
    engine.terminate();
    computePool.terminate();
  });
}

/** Runs `action` once per click a click counter counts, clicks before this module loaded included. */
function onClicks(counter, action) {
  let seen = 0;
  return counter.subscribe((count) => {
    for (; seen < count; seen++) action();
  });
}
