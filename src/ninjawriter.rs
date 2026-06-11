use std::{
    collections::BTreeSet,
    fmt::{Display, Write},
};

/*
 * Model
 */

#[derive(Default, Debug)]
struct UniqueNames {
    names: BTreeSet<String>,
}

#[derive(Debug)]
struct NinjaVar {
    name: String,
    args: Vec<String>,
}

#[derive(Debug, Default)]
pub struct NinjaRule {
    name: String,
    vars: Vec<NinjaVar>,
}

#[derive(Debug, PartialEq)]
pub struct NinjaRuleRef(String);

#[derive(Debug, Default)]
pub struct NinjaBuild {
    rule: String,
    outputs: Vec<String>,
    inputs: Vec<String>,
    deps: Vec<String>,
    vars: Vec<NinjaVar>,
    is_default: bool,
}

#[derive(Debug, Default)]
pub struct NinjaFile {
    rule_names: UniqueNames,
    rules: Vec<NinjaRule>,
    builds: Vec<NinjaBuild>,
}

/*
 * Display
 */

fn ninja_indent(f: &mut std::fmt::Formatter<'_>, indent: i32) -> std::fmt::Result {
    for _ in 0..indent {
        f.write_str("  ")?;
    }
    Ok(())
}

fn ninja_esc_string(f: &mut std::fmt::Formatter<'_>, indent: i32, input: &str) -> std::fmt::Result {
    for c in input.chars() {
        match c {
            '$' => write!(f, "$$")?,
            '\n' => {
                write!(f, "$\n")?;
                ninja_indent(f, indent)?;
            }
            ':' => write!(f, "$:")?,
            ' ' => write!(f, "$ ")?,
            c => f.write_char(c)?,
        }
    }
    Ok(())
}

impl NinjaVar {
    fn write(&self, indent: i32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ninja_indent(f, indent)?;
        ninja_esc_string(f, indent + 1, &self.name)?;
        f.write_str(" =")?;
        for arg in self.args.iter() {
            f.write_char(' ')?;
            ninja_esc_string(f, indent + 1, arg)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

impl Display for NinjaRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("rule ")?;
        ninja_esc_string(f, 1, &self.name)?;
        f.write_char('\n')?;
        for var in self.vars.iter() {
            var.write(1, f)?;
        }
        f.write_char('\n')?;
        Ok(())
    }
}

impl Display for NinjaBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("build")?;
        for outp in self.outputs.iter() {
            f.write_char(' ')?;
            ninja_esc_string(f, 1, outp)?;
        }
        f.write_str(": ")?;
        ninja_esc_string(f, 1, &self.rule)?;
        for inp in self.inputs.iter() {
            f.write_char(' ')?;
            ninja_esc_string(f, 1, inp)?;
        }
        if !self.deps.is_empty() {
            f.write_str(" |")?;
            for dep in self.deps.iter() {
                f.write_char(' ')?;
                ninja_esc_string(f, 1, dep)?;
            }
        }
        f.write_char('\n')?;
        for var in self.vars.iter() {
            var.write(1, f)?;
        }
        f.write_char('\n')?;
        if self.is_default {
            f.write_str("default")?;
            for outp in self.outputs.iter() {
                f.write_char(' ')?;
                ninja_esc_string(f, 1, outp)?;
            }
            f.write_char('\n')?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Display for NinjaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rule in self.rules.iter() {
            rule.fmt(f)?;
        }
        for build in self.builds.iter() {
            build.fmt(f)?;
        }
        Ok(())
    }
}

/*
 * Tools
 */
impl UniqueNames {
    fn get(&mut self, name: impl ToString) -> String {
        let name = name.to_string();
        if self.names.insert(name.clone()) {
            return name;
        }

        for idx in 1.. {
            let indexed_name = format!("{}{}", name, idx);
            if self.names.insert(indexed_name.clone()) {
                return indexed_name;
            }
        }
        unreachable!()
    }
}

/*
 * Construction
 */

impl NinjaRule {
    fn new(name: impl ToString) -> Self {
        let mut rule: NinjaRule = Default::default();
        rule.name = name.to_string();
        rule
    }

    pub fn var(&mut self, name: impl ToString, args: Vec<String>) -> &mut Self {
        self.vars.push(NinjaVar {
            name: name.to_string(),
            args,
        });
        self
    }

    pub fn as_ref(&self) -> NinjaRuleRef {
        NinjaRuleRef(self.name.clone())
    }
}

impl NinjaBuild {
    fn new(rule: &NinjaRuleRef) -> Self {
        let mut build: NinjaBuild = Default::default();
        build.rule = rule.0.clone();
        build.is_default = false;
        build
    }

    pub fn output(&mut self, name: impl ToString) -> &mut Self {
        self.outputs.push(name.to_string());
        self
    }

    pub fn input(&mut self, name: impl ToString) -> &mut Self {
        self.inputs.push(name.to_string());
        self
    }

    pub fn dep(&mut self, name: impl ToString) -> &mut Self {
        self.deps.push(name.to_string());
        self
    }

    pub fn var(&mut self, name: impl ToString, args: Vec<String>) -> &mut Self {
        self.vars.push(NinjaVar {
            name: name.to_string(),
            args,
        });
        self
    }

    pub fn set_default(&mut self) -> &mut Self {
        self.is_default = true;
        self
    }
}

impl NinjaFile {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rule(&mut self, name: impl ToString) -> &mut NinjaRule {
        let unique_name = self.rule_names.get(name);
        self.rules.push(NinjaRule::new(unique_name));
        self.rules.last_mut().unwrap()
    }

    pub fn build(&mut self, rule: &NinjaRuleRef) -> &mut NinjaBuild {
        self.builds.push(NinjaBuild::new(rule));
        self.builds.last_mut().unwrap()
    }
}

/*
 * Tests
 */

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! lines (
        ($line:expr) => ($line);
        ($line:expr, $($rest:expr),+) => (concat!($line, "\n", lines!($($rest),+)));
        () => ("");
    );

    #[test]
    fn test_write_rule() {
        let mut rule = NinjaRule::new("test");
        rule.var("deps", vec!["boll".into(), "something".into()])
            .var("something", vec!["stuff".into(), "stuff".into()]);
        assert_eq!(
            format!("{}", rule).as_str(),
            lines! {
                "rule test",
                "  deps = boll something",
                "  something = stuff stuff",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_esc_string() {
        let mut rule = NinjaRule::new("r$a");
        rule.var("a b", vec!["a$b".into(), "a b".into()])
            .var("b", vec!["a\nb".into(), "a:b".into()]);
        assert_eq!(
            format!("{}", rule).as_str(),
            lines! {
                "rule r$$a",
                "  a$ b = a$$b a$ b",
                "  b = a$",
                "    b a$:b",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_build() {
        let mut rule = NinjaRule::new("r$a");
        rule.var("a b", vec!["a$b".into(), "a b".into()])
            .var("b", vec!["a\nb".into(), "a:b".into()]);
        let mut build = NinjaBuild::new(&rule.as_ref());
        build
            .input("boll")
            .input("hej")
            .output("dest")
            .output("destb")
            .var("tjo", vec!["xx".into()]);
        let output = format!("{}{}", rule, build);
        assert_eq!(
            output.as_str(),
            lines! {
                "rule r$$a",
                "  a$ b = a$$b a$ b",
                "  b = a$",
                "    b a$:b",
                "",
                "build dest destb: r$$a boll hej",
                "  tjo = xx",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_build_deps() {
        let rule = NinjaRule::new("rule");
        let mut build = NinjaBuild::new(&rule.as_ref());
        build.input("in").output("out").dep("dep");
        let output = format!("{}{}", rule, build);
        assert_eq!(
            output.as_str(),
            lines! {
                "rule rule",
                "",
                "build out: rule in | dep",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_build_default() {
        let rule = NinjaRule::new("rule");
        let mut build = NinjaBuild::new(&rule.as_ref());
        build.input("in").output("out").set_default();
        let output = format!("{}{}", rule, build);
        assert_eq!(
            output.as_str(),
            lines! {
                "rule rule",
                "",
                "build out: rule in",
                "",
                "default out",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_file() {
        let mut file = NinjaFile::new();

        let rule1 = file.rule("test1").var("x", vec!["stuff".into()]).as_ref();
        let _rule2 = file.rule("test2").var("y", vec!["stuff".into()]).as_ref();

        file.build(&rule1)
            .input("in1_1")
            .input("in1_2")
            .output("out1");

        assert_eq!(
            format!("{}", file),
            lines! {
                "rule test1",
                "  x = stuff",
                "",
                "rule test2",
                "  y = stuff",
                "",
                "build out1: test1 in1_1 in1_2",
                "",
                ""
            }
        );
    }

    #[test]
    fn test_file_unique_rules() {
        let mut file = NinjaFile::new();

        assert_eq!(file.rule("test").as_ref(), NinjaRuleRef("test".into()));
        assert_eq!(file.rule("test").as_ref(), NinjaRuleRef("test1".into()));
        assert_eq!(file.rule("x").as_ref(), NinjaRuleRef("x".into()));
        assert_eq!(file.rule("test").as_ref(), NinjaRuleRef("test2".into()));
        assert_eq!(file.rule("x").as_ref(), NinjaRuleRef("x1".into()));
        assert_eq!(file.rule("test").as_ref(), NinjaRuleRef("test3".into()));
        assert_eq!(file.rule("test").as_ref(), NinjaRuleRef("test4".into()));
        assert_eq!(file.rule("x").as_ref(), NinjaRuleRef("x2".into()));
    }

    #[test]
    fn test_ref_unique_name() {
        let mut file = NinjaFile::new();

        let _rule = file.rule("test").as_ref();
        let rule1 = file.rule("test").as_ref();
        let rule2 = file.rule("test").as_ref();

        file.build(&rule1).output("out1").set_default();
        file.build(&rule2).output("out2");

        assert_eq!(
            format!("{}", file),
            lines! {
                "rule test",
                "",
                "rule test1",
                "",
                "rule test2",
                "",
                "build out1: test1",
                "",
                "default out1",
                "",
                "build out2: test2",
                "",
                ""
            }
        );
    }
}
