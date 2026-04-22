#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../../.."

test -f src/application/read_model/transcript.rs
test -f src/application/read_model/forensics.rs
test -f src/application/read_model/manifold.rs
test -f src/application/read_model/projection.rs

test ! -f src/domain/model/transcript.rs
test ! -f src/domain/model/forensics.rs
test ! -f src/domain/model/manifold.rs
test ! -f src/domain/model/projection.rs

rg -q '^pub mod read_model;$' src/application/mod.rs
rg -q 'pub use self::read_model::' src/application/mod.rs
rg -q 'pub use crate::application::read_model::' src/domain/model/mod.rs

! rg -q '^pub mod transcript;$' src/domain/model/mod.rs
! rg -q '^pub mod forensics;$' src/domain/model/mod.rs
! rg -q '^pub mod manifold;$' src/domain/model/mod.rs
! rg -q '^pub mod projection;$' src/domain/model/mod.rs
