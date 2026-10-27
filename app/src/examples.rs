//! The messages the page offers to start from.

/// An example message.
#[derive(Clone, PartialEq)]
pub(crate) struct Example {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) text: &'static str,
}

pub(crate) const EXAMPLES: [Example; 3] = [
    Example {
        id: "colors",
        label: "A question",
        text: "Name the three primary colors of light, in one sentence.",
    },
    Example {
        id: "trains",
        label: "A puzzle",
        text: "A train leaves at 9:40 and arrives at 13:15. How long is the trip?",
    },
    Example {
        id: "haiku",
        label: "A haiku",
        text: "Write a haiku about a browser tab that thinks.",
    },
];
