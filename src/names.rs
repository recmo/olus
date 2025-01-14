use {
    crate::files::{FileId, Span},
    ariadne::Label,
    std::ops::{Index, Range},
};

pub struct Names {
    binders: Vec<Binder>,
}

pub struct Binder {
    span:   Span,
    suffix: usize,
}

pub struct BinderId(usize);

pub struct Reference {
    span:   Span,
    binder: BinderId,
}

impl Index<BinderId> for Names {
    type Output = Binder;

    fn index(&self, id: BinderId) -> &Self::Output {
        &self.binders[id.0]
    }
}

impl Binder {
    fn label(&self) -> Label<Span> {
        Label::new(self.span)
    }
}

impl Reference {
    fn label(&self) -> Label<Span> {
        Label::new(self.span)
    }
}
