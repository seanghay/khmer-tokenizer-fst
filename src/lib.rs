/// Pulled from
/// https://github.com/meilisearch/charabia/blob/main/charabia/src/segmenter/khmer.rs
/// https://github.com/meilisearch/charabia/blob/main/charabia/src/segmenter/utils.rs
use fst::raw::{Fst, Output};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

fn find_longest_prefix(fst: &Fst<&[u8]>, value: &[u8]) -> Option<(u64, usize)> {
    let mut node = fst.root();
    let mut out = Output::zero();
    let mut last_match = None;
    for (i, &b) in value.iter().enumerate() {
        if let Some(trans_index) = node.find_input(b) {
            let t = node.transition(trans_index);
            node = fst.node(t.addr);
            out = out.cat(t.out);
            if node.is_final() {
                last_match = Some((out.cat(node.final_output()).value(), i + 1));
            }
        } else {
            return last_match;
        }
    }
    last_match
}

fn segment_text<'o>(
    fst: &'o Fst<&[u8]>,
    mut to_segment: &'o str,
) -> Box<dyn Iterator<Item = &'o str> + 'o> {
    let iter = std::iter::from_fn(move || {
        // if we reach the end of the text, we return None.
        if to_segment.is_empty() {
            return None;
        }

        let length = match find_longest_prefix(fst, to_segment.as_bytes()) {
            Some((_, length)) => length,
            None => {
                // if no sequence matches, we return the next character as a lemma.
                let first = to_segment.chars().next().unwrap();
                first.len_utf8()
            }
        };
        let (left, right) = to_segment.split_at(length);
        to_segment = right;
        Some(left)
    });

    Box::new(iter)
}

static FST: Lazy<Fst<&[u8]>> = Lazy::new(|| Fst::new(&include_bytes!("./words.fst")[..]).unwrap());

#[wasm_bindgen]
pub fn segment(text: &str) -> Vec<String> {
    segment_text(&FST, text)
        .map(String::from)
        .collect::<Vec<String>>()
}
