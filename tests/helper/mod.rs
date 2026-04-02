/*
 * SPDX-FileCopyrightText: 2021 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 hoijui <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
use std::path::{Path, PathBuf};

fn proj_root_dir() -> PathBuf {
    Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}

pub fn benches_dir() -> PathBuf {
    proj_root_dir().join("benches")
}

pub fn target_dir() -> PathBuf {
    proj_root_dir().join("target")
}
