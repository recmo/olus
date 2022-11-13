use super::syntax::{Argument, Block, Call, Def, Identifier, Line, Proc, Root};
use std::io::{Error, Write};

pub fn unparse<W: Write>(writer: &mut W, node: Root) -> Result<(), Error> {
    Unparser::new(writer).unparse_root(node)
}

struct Unparser<W: Write> {
    writer: W,
    indent: usize,
}

impl<W: Write> Unparser<W> {
    fn new(writer: W) -> Self {
        Self { writer, indent: 0 }
    }

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
        self.unparse_identifier(proc.name())?;
        for parameter in proc.parameters() {
            write!(self.writer, " ")?;
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
        write!(self.writer, "{}", identifier.text())
    }

    fn unparse_argument(&mut self, argument: Argument) -> Result<(), Error> {
        match argument {
            Argument::Identifier(identifier) => self.unparse_identifier(identifier)?,
            Argument::Group(call) => {
                write!(self.writer, "(")?;
                // ...
                write!(self.writer, ")")?;
            }
        }
        Ok(())
    }
}
