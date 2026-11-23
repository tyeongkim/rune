use super::*;

/// Where an element came from in the view code (debug builds only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElementSource {
    /// The call that built it: [`el`], [`row`], [`col`], [`text`] and the other element
    /// constructors, or the nearest caller above them marked `#[track_caller]`.
    pub built: Option<&'static std::panic::Location<'static>>,
    /// Its [`El::key`] call.
    pub key: Option<&'static std::panic::Location<'static>>,
}

impl<S: 'static> Runtime<S> {
    /// Where the element at `path` (an [`ElementInfo::path`]) was built and keyed. Empty when it
    /// isn't on screen, or in builds without debug assertions.
    pub fn element_source(&self, path: &str) -> ElementSource {
        self.core().element_source(path)
    }
}

impl RuntimeCore {
    #[cfg(debug_assertions)]
    fn element_source(&self, path: &str) -> ElementSource {
        self.tree.index_of_path(path).and_then(|i| self.tree.nodes.get(i)).map_or_else(ElementSource::default, |n| ElementSource {
            built: n.built_source,
            key: n.key_source,
        })
    }

    #[cfg(not(debug_assertions))]
    fn element_source(&self, _: &str) -> ElementSource {
        ElementSource::default()
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use crate::{El, Env, Runtime, col, el, text};

    const VIEW_LINE: u32 = line!() + 3;

    fn view(_: &(), _: &mut crate::Cx<()>) -> El {
        col().key("outer").child(el().key("inner").w(10.0).h(10.0)).child(text("unkeyed"))
    }

    #[track_caller]
    fn helper() -> El {
        el().w(4.0).h(4.0)
    }

    const HELPER_LINE: u32 = line!() + 3;

    fn helper_view(_: &(), _: &mut crate::Cx<()>) -> El {
        col().child(helper().key("helped"))
    }

    fn sources(view: fn(&(), &mut crate::Cx<()>) -> El) -> Vec<(Option<String>, super::ElementSource)> {
        let mut rt = Runtime::new((), view, (100.0, 100.0), Env::new(false, 1.0));
        rt.frame(0.0);
        rt.elements().into_iter().filter(|e| e.parent.is_some()).map(|e| (e.key.clone(), rt.element_source(&e.path))).collect()
    }

    #[test]
    fn element_source_names_the_build_and_key_calls() {
        let here = |l: Option<&'static std::panic::Location<'static>>| l.map(|l| (l.file(), l.line()));
        for (key, source) in sources(view) {
            assert_eq!(here(source.built), Some((file!(), VIEW_LINE)), "{key:?}");
            let keyed = key.is_some().then_some((file!(), VIEW_LINE));
            assert_eq!(here(source.key), keyed, "{key:?}");
        }
    }

    #[test]
    fn a_track_caller_helper_reports_its_caller() {
        let (_, source) = sources(helper_view).into_iter().find(|(k, _)| k.as_deref() == Some("helped")).expect("helped");
        assert_eq!(source.built.map(|l| l.line()), Some(HELPER_LINE));
    }
}
