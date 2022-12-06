use crate::{
    front::Resolution,
    parser::syntax::{Argument, Block, Call, Def, Identifier, Line, Proc, Root},
};
use ariadne::{Color, ColorGenerator, Fmt};
use std::io::{Error, Write};

pub fn unparse<'a, W: Write>(
    writer: &mut W,
    root: Root,
    resolution: Option<&'a Resolution>,
) -> Result<(), Error> {
    let colors = if let Some(res) = resolution {
        let mut color_generator = ColorGenerator::new();
        res.binders
            .iter()
            .map(|_| color_generator.next())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut unparser = Unparser {
        writer,
        root,
        resolution,
        colors,
        indent: 0,
    };
    unparser.unparse_root()
}
struct Unparser<'a, W: Write> {
    writer:     W,
    root:       Root,
    resolution: Option<&'a Resolution>,
    colors:     Vec<Color>,
    indent:     usize,
}

impl<'a, W: Write> Unparser<'a, W> {
    fn unparse_root(&mut self) -> Result<(), Error> {
        for line in self.root.lines() {
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
        self.unparse_proc(def.procedure())?;
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
            .and_then(|resolution| resolution.lookup(&identifier, &self.root));

        let color = Color::Black;

        let unbound = reference.is_none();
        let binds = Some(&identifier) == reference.as_ref();
        if unbound {
            write!(self.writer, "{}", Color::Red.paint(identifier.text()))
        } else if binds {
            write!(self.writer, "{}", color.paint(identifier.text()).bold())
        } else {
            write!(self.writer, "{}", color.paint(identifier.text()))
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
