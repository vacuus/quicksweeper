use itertools::Itertools;
use once_cell::sync::Lazy;

use crate::minefield::FieldShape;

pub static FIELDS: Lazy<Vec<FieldShape>> = Lazy::new(|| {
    std::fs::read_dir("assets/fields")
        .map(|dir| {
            dir.flat_map(|entry| {
                entry
                    .map_err(|_| ())
                    .and_then(|u| {
                        FieldShape::try_from(std::fs::read(u.path()).unwrap().as_slice())
                            .map_err(|_| ())
                    })
                    .ok()
            })
            .collect_vec()
        })
        .unwrap_or(Vec::new())
});
