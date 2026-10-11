//! Independent callback failure domains. Ordinary arithmetic performs no
//! attempt/TLS work; existing contour-only arithmetic uses its original fence.
#[derive(Clone, Copy, Default)]
pub(super) struct Modes {
    pub contour: bool,
    pub algebraic: bool,
}
impl Modes {
    pub fn isolated<R>(self, run: impl FnOnce() -> R) -> (R, Option<String>) {
        match (self.contour, self.algebraic) {
            (false, false) => (run(), None),
            (true, false) => crate::contour::functions::dynamic::isolated_attempt(run),
            (false, true) => {
                let (value, error) = super::algebraic::isolated_attempt(run);
                (value, error.map(|e| format!("algebraic callback: {e}")))
            }
            (true, true) => {
                let ((value, root), contour) =
                    crate::contour::functions::dynamic::isolated_attempt(|| {
                        super::algebraic::isolated_attempt(run)
                    });
                (
                    value,
                    match (root, contour) {
                        (Some(root), Some(contour)) => Some(format!(
                            "algebraic callback: {root}; contour callback: {contour}"
                        )),
                        (Some(root), None) => Some(format!("algebraic callback: {root}")),
                        (None, contour) => contour,
                    },
                )
            }
        }
    }
}
