use crate::{
    front::Resolution,
    parser::syntax::{Argument, Block, Call, Def, Identifier, Line, Proc, Root},
};
use owo_colors::{DynColors, OwoColorize};
use std::io::{Error, Write};

pub fn unparse<W: Write>(
    writer: &mut W,
    node: Root,
    resolution: Option<Resolution>,
) -> Result<(), Error> {
    let mut unparser = Unparser {
        writer,
        resolution,
        indent: 0,
    };
    unparser.unparse_root(node)
}

// Deterministic pseudo-random color for identifier
fn color(identifier: &Identifier) -> DynColors {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let grad = colorgrad::rainbow();
    let hash = {
        let mut hasher = DefaultHasher::new();
        identifier.text().hash(&mut hasher);
        identifier.syntax().text_range().hash(&mut hasher);
        hasher.finish() as f64 / std::u64::MAX as f64
    };
    let color = grad.at(hash).to_rgba8();
    DynColors::Rgb(color[0], color[1], color[2])
}

struct Unparser<W: Write> {
    writer:     W,
    resolution: Option<Resolution>,
    indent:     usize,
}

impl<W: Write> Unparser<W> {
    fn unparse_root(&mut self, root: Root) -> Result<(), Error> {
        for line in root.lines() {
            self.unparse_line(line)?;
            writeln!(self.writer)?;
        }
        Ok(())
    }

    fn unparse_line(&mut self, line: Line) -> Result<(), Error> {
        write!(self.writer, "{:indent$}", "", indent = self.indent)?;
        match line.clone() {
            Line::Def(def) => self.unparse_def(def)?,
            Line::Call(call) => self.unparse_call(call)?,
        };
        writeln!(self.writer)?;
        if let Some(block) = line.block() {
            self.indent += 4;
            self.unparse_block(block)?;
            self.indent -= 4;
        }
        Ok(())
    }

    fn unparse_def(&mut self, def: Def) -> Result<(), Error> {
        self.unparse_proc(def.proc())?;
        if let Some(call) = def.call() {
            write!(self.writer, " ")?;
            self.unparse_call(call)?;
        }
        Ok(())
    }

    fn unparse_proc(&mut self, proc: Proc) -> Result<(), Error> {
        for (i, parameter) in proc.identifiers().enumerate() {
            if i > 0 {
                write!(self.writer, " ")?;
            }
            self.unparse_identifier(parameter)?;
        }
        write!(self.writer, ":")?;
        Ok(())
    }

    fn unparse_call(&mut self, call: Call) -> Result<(), Error> {
        for (i, argument) in call.arguments().enumerate() {
            if i > 0 {
                write!(self.writer, " ")?;
            }
            self.unparse_argument(argument)?;
        }
        Ok(())
    }

    fn unparse_block(&mut self, block: Block) -> Result<(), Error> {
        for line in block.lines() {
            self.unparse_line(line)?;
        }
        Ok(())
    }

    fn unparse_identifier(&mut self, identifier: Identifier) -> Result<(), Error> {
        let reference = self
            .resolution
            .as_ref()
            .and_then(|resolution| resolution.lookup(&identifier));
        let color = reference.map(color).unwrap_or(DynColors::Rgb(0, 0, 0));
        let unbound = reference.is_none();
        let binds = Some(&identifier) == reference;
        if unbound {
            write!(self.writer, "{}", identifier.text().on_bright_red())
        } else if binds {
            write!(self.writer, "{}", identifier.text().color(color).bold())
        } else {
            write!(self.writer, "{}", identifier.text().color(color))
        }
    }

    fn unparse_argument(&mut self, argument: Argument) -> Result<(), Error> {
        match argument {
            Argument::Identifier(identifier) => self.unparse_identifier(identifier)?,
            Argument::Group(group) => {
                write!(self.writer, "(")?;
                if let Some(def) = group.def() {
                    self.unparse_def(def)?;
                }
                if let Some(call) = group.call() {
                    self.unparse_call(call)?;
                }
                write!(self.writer, ")")?;
            }
            Argument::String(string) => write!(self.writer, "“{}”", string.value())?,
            Argument::Number(number) => write!(self.writer, "{}", number.text())?,
        }
        Ok(())
    }
}
