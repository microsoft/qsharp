// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display, Write};

fn with_indentation<T>(f: &mut T) -> indenter::Indented<'_, T>
where
    T: fmt::Write,
{
    let indent = indenter::indented(f);
    set_indentation(indent, 1)
}

fn set_indentation<T>(indent: indenter::Indented<'_, T>, level: usize) -> indenter::Indented<'_, T>
where
    T: fmt::Write,
{
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        3 => indent.with_str("            "),
        _ => unimplemented!("indentation level not supported"),
    }
}

fn write_list<'write, 'itemref, 'item, T, I>(f: &'write mut impl Write, vals: I) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    let mut iter = vals.into_iter().peekable();
    if iter.peek().is_none() {
        write!(f, " <empty>")
    } else {
        for elt in iter {
            write!(f, "\n{elt}")?;
        }
        Ok(())
    }
}

pub(super) fn write_indented_list<'write, 'itemref, 'item, T, I>(
    f: &'write mut impl Write,
    vals: I,
) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    let mut iter = vals.into_iter().peekable();
    if iter.peek().is_none() {
        write!(f, " <empty>")
    } else {
        let mut indent = with_indentation(f);
        for elt in iter {
            write!(indent, "\n{elt}")?;
        }
        Ok(())
    }
}

pub(super) fn write_header(f: &mut impl Write, name: &str, span: super::Span) -> fmt::Result {
    write!(f, "{name} {span}:")
}

pub(super) fn writeln_header(f: &mut impl Write, name: &str, span: super::Span) -> fmt::Result {
    writeln!(f, "{name} {span}:")
}

pub(super) fn write_field<T: Display>(
    f: &mut impl Write,
    field_name: &str,
    val: &T,
) -> fmt::Result {
    let mut indent = with_indentation(f);
    write!(indent, "{field_name}: {val}")
}

pub(super) fn writeln_field<T: Display>(
    f: &mut impl Write,
    field_name: &str,
    val: &T,
) -> fmt::Result {
    write_field(f, field_name, val)?;
    writeln!(f)
}

pub(super) fn write_opt_field<T: Display>(
    f: &mut impl Write,
    field_name: &str,
    opt_val: Option<&T>,
) -> fmt::Result {
    if let Some(val) = opt_val {
        write_field(f, field_name, val)
    } else {
        write_field(f, field_name, &"<none>")
    }
}

pub(super) fn writeln_opt_field<T: Display>(
    f: &mut impl Write,
    field_name: &str,
    opt_val: Option<&T>,
) -> fmt::Result {
    write_opt_field(f, field_name, opt_val)?;
    writeln!(f)
}

pub(super) fn write_list_field<'write, 'itemref, 'item, T, I>(
    f: &mut impl Write,
    field_name: &str,
    vals: I,
) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    let mut indent = with_indentation(f);
    write!(indent, "{field_name}:")?;
    let mut indent = set_indentation(indent, 2);
    write_list(&mut indent, vals)
}

pub(super) fn writeln_list_field<'write, 'itemref, 'item, T, I>(
    f: &mut impl Write,
    field_name: &str,
    vals: I,
) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    write_list_field(f, field_name, vals)?;
    writeln!(f)
}

pub(super) fn write_opt_list_field<'write, 'itemref, 'item, T, I>(
    f: &mut impl Write,
    field_name: &str,
    opt_vals: Option<I>,
) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    if let Some(vals) = opt_vals {
        write_list_field(f, field_name, vals)
    } else {
        let mut indent = with_indentation(f);
        write!(indent, "{field_name}: <none>")
    }
}

pub(super) fn writeln_opt_list_field<'write, 'itemref, 'item, T, I>(
    f: &mut impl Write,
    field_name: &str,
    opt_vals: Option<I>,
) -> fmt::Result
where
    'item: 'itemref,
    T: Display + 'item,
    I: IntoIterator<Item = &'itemref T>,
{
    write_opt_list_field(f, field_name, opt_vals)?;
    writeln!(f)
}
