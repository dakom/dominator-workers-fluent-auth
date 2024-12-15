pub use crate::{
    api_ext::{self, ApiBothExt, ApiEmptyExt, ApiReqExt, ApiResExt},
    auth::AUTH,
    config::*,
    error::*,
    get_text,
    locale::*,
    route::*,
    text_args,
    theme::{color::*, misc::*, typography::*, z_index::*},
};
pub use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
pub use awsm_web::prelude::*;
use dominator::DomBuilder;
pub use dominator::{
    apply_methods, attrs, class, clone, events, fragment, html, link, pseudo, styles, svg,
    with_node, Dom, Fragment,
};
pub use futures_signals::{
    map_ref,
    signal::{Mutable, Signal, SignalExt},
    signal_vec::{MutableVec, SignalVec, SignalVecExt},
};
pub use serde::{Deserialize, Serialize};
pub use shared::frontend::route::*;
pub use std::sync::{Arc, LazyLock, Mutex, RwLock};
pub use wasm_bindgen::prelude::*;

pub type MixinStub<T> = fn(DomBuilder<T>) -> DomBuilder<T>;
//pub type MixinFn<T, F> = F;

pub trait MixinFnOnce<T>: FnOnce(DomBuilder<T>) -> DomBuilder<T> {}
impl<T, F> MixinFnOnce<T> for F where F: FnOnce(DomBuilder<T>) -> DomBuilder<T> {}

pub trait MixinFn<T>: Fn(DomBuilder<T>) -> DomBuilder<T> {}
impl<T, F> MixinFn<T> for F where F: Fn(DomBuilder<T>) -> DomBuilder<T> {}
