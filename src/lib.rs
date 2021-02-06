/*!

# HOI4 Save

HOI4 Save is a library to ergonomically work with Hearts of Iron IV saves (plaintext + binary).

```rust,ignore
use hoi4save::{Hoi4Extractor, Encoding};
use std::io::Cursor;

let data = std::fs::read("assets/saves/1.10-normal-text.hoi4")?;
let (save, encoding) = Hoi4Extractor::extract_save(&data[..])?;
assert_eq!(encoding, Encoding::Plaintext);
assert_eq!(save.player, String::from("FRA"));
```

The HOI4 binary format can be converted to plaintext with the help of `hoi4save::Melter`:

```rust,ignore
let data = std::fs::read("assets/saves/1.10-ironman.hoi4")?;
let (melted, _unknown_tokens) = hoi4save::Melter::new()
    .with_on_failed_resolve(hoi4save::FailedResolveStrategy::Stringify)
    .melt(&data[..])?;
```

## Binary Saves

By default, binary saves will not be decoded properly.

To enable support, one must supply an environment variable
(`HOI4_IRONMAN_TOKENS`) that points to a newline delimited
text file of token descriptions. For instance:

```ignore
0xffff my_test_token
0xeeee my_test_token2
```

In order to comply with legal restrictions, I cannot share the list of
tokens. I am also restricted from divulging how the list of tokens can be derived.

*/

mod date;
mod errors;
mod extraction;
mod flavor;
mod melt;
pub mod models;
mod tokens;

pub use date::*;
pub use errors::*;
pub use extraction::*;
pub use jomini::FailedResolveStrategy;
pub use melt::*;
