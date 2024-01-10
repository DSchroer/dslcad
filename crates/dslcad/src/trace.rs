#[macro_export]
macro_rules! elapsed {
    ($tag: expr, $action: expr) => {{
        let timer = std::time::Instant::now();
        let ret = $action;
        log::trace!("{} in {}ms", $tag, timer.elapsed().as_millis());
        ret
    }};
}
