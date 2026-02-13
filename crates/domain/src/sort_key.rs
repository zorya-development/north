use fractional_index::FractionalIndex;

pub fn sort_key_between(above: Option<&str>, below: Option<&str>) -> String {
    match (above, below) {
        (None, None) => FractionalIndex::default().to_string(),
        (Some(a), None) => {
            let fi = FractionalIndex::from_string(a).unwrap_or_default();
            FractionalIndex::new_after(&fi).to_string()
        }
        (None, Some(b)) => {
            let fi = FractionalIndex::from_string(b).unwrap_or_default();
            FractionalIndex::new_before(&fi).to_string()
        }
        (Some(a), Some(b)) => {
            let fa = FractionalIndex::from_string(a).unwrap_or_default();
            let fb = FractionalIndex::from_string(b).unwrap_or_default();
            FractionalIndex::new_between(&fa, &fb)
                .unwrap_or_else(|| FractionalIndex::new_after(&fa))
                .to_string()
        }
    }
}

pub fn sort_key_after(last: Option<&str>) -> String {
    sort_key_between(last, None)
}
