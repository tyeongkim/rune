use super::*;

impl<S: 'static> Runtime<S> {
    /// Where the element at `path` (an [`ElementInfo::path`]) got its key: the file, line and
    /// column of its [`El::key`] call. `None` when it has no key, isn't on screen, or in builds
    /// without debug assertions.
    pub fn element_source(&self, path: &str) -> Option<&'static std::panic::Location<'static>> {
        self.core().element_source(path)
    }
}

impl RuntimeCore {
    #[cfg(debug_assertions)]
    fn element_source(&self, path: &str) -> Option<&'static std::panic::Location<'static>> {
        self.tree.nodes.get(self.tree.index_of_path(path)?)?.key_source
    }

    #[cfg(not(debug_assertions))]
    fn element_source(&self, _: &str) -> Option<&'static std::panic::Location<'static>> {
        None
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use crate::{El, Env, Runtime, col, el};

    const VIEW_LINE: u32 = line!() + 3;

    fn view(_: &(), _: &mut crate::Cx<()>) -> El {
        col().key("outer").child(el().key("inner").w(10.0).h(10.0)).child(el().w(4.0).h(4.0))
    }

    #[test]
    fn element_source_names_the_key_call() {
        let mut rt = Runtime::new((), view, (100.0, 100.0), Env::new(false, 1.0));
        rt.frame(0.0);
        let elements = rt.elements();
        let path = |key: &str| elements.iter().find(|e| e.key.as_deref() == Some(key)).expect(key).path.clone();
        for key in ["outer", "inner"] {
            let source = rt.element_source(&path(key)).expect(key);
            assert_eq!((source.file(), source.line()), (file!(), VIEW_LINE), "{key}");
        }
        let unkeyed = elements.iter().find(|e| e.key.is_none() && e.parent.is_some()).expect("unkeyed child");
        assert!(rt.element_source(&unkeyed.path).is_none());
    }
}
