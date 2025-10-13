/*
 * SPDX-FileCopyrightText: 2021 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 hoijui <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
use std::path::{Path, PathBuf};

pub fn benches_dir() -> PathBuf {
    Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benches")
}
