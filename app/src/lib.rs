//! The Spark-X2.6 page. The conversation, the next message (its text and how to reply) and the
//! live measurements are signals here; `web/app.js` runs the engine worker and reports back
//! through `engine` events.

mod app;
mod examples;
mod markdown;
