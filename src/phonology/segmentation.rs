#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpokenForm {
    pub label: String,
    pub pronunciation: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Segmentation {
    Impossible,
    Unique(Vec<String>),
    Ambiguous { first: Vec<String>, second: Vec<String> },
}

pub fn segment_spoken_stream(stream: &str, forms: &[SpokenForm]) -> Segmentation {
    if stream.is_empty() {
        return Segmentation::Unique(Vec::new());
    }
    let forms = forms
        .iter()
        .filter(|form| !form.pronunciation.is_empty())
        .collect::<Vec<_>>();
    let mut memo = vec![None; stream.len() + 1];
    let paths = paths_from(stream, 0, &forms, &mut memo);
    match paths.as_slice() {
        [] => Segmentation::Impossible,
        [only] => Segmentation::Unique(only.clone()),
        [first, second, ..] => Segmentation::Ambiguous {
            first: first.clone(),
            second: second.clone(),
        },
    }
}

fn paths_from(
    stream: &str,
    offset: usize,
    forms: &[&SpokenForm],
    memo: &mut [Option<Vec<Vec<String>>>],
) -> Vec<Vec<String>> {
    if offset == stream.len() {
        return vec![Vec::new()];
    }
    if let Some(paths) = &memo[offset] {
        return paths.clone();
    }

    let remainder = &stream[offset..];
    let mut results = Vec::new();
    for form in forms {
        if !remainder.starts_with(&form.pronunciation) {
            continue;
        }
        let next = offset + form.pronunciation.len();
        if !stream.is_char_boundary(next) {
            continue;
        }
        for suffix in paths_from(stream, next, forms, memo) {
            let mut path = Vec::with_capacity(suffix.len() + 1);
            path.push(form.label.clone());
            path.extend(suffix);
            results.push(path);
            if results.len() >= 2 {
                memo[offset] = Some(results.clone());
                return results;
            }
        }
    }
    memo[offset] = Some(results.clone());
    results
}
