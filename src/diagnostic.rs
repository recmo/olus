use {
    crate::Span,
    ariadne::{Color, Report, ReportKind},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub message: String,
    pub span:    Span,
}

impl Diagnostic {
    #[must_use]
    pub fn report(&self) -> Report<Span> {
        self.span
            .report(ReportKind::Error)
            .with_message(self.message.clone())
            .with_label(
                self.span
                    .label()
                    .with_color(Color::Red)
                    .with_message(self.message.clone()),
            )
            .finish()
    }
}
