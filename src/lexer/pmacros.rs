use bitflags::bitflags;
use hashbrown::HashMap;
use std::cell::Cell;
use std::fmt;

use super::lexer::Lexer;
use super::macro_args::{MacroDefArg, MacroNode};
use super::preprocessor::MacroToken;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum IfState {
    Eval,
    Skip,
    SkipAndSwitch,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum IfKind {
    If,
    Ifdef,
    Ifndef,
}

#[derive(Clone, Debug)]
pub(crate) struct PContext {
    macros: HashMap<String, Macro>,
    if_stack: Vec<IfState>,
}

impl Default for PContext {
    fn default() -> Self {
        Self {
            macros: HashMap::default(),
            if_stack: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct MacroObject {
    out: Vec<u8>,
    has_id: bool,
    in_use: Cell<bool>,
}

#[derive(Clone)]
pub(crate) struct MacroFunction {
    out: Vec<u8>,
    actions: Vec<Action>,
    n_args: usize,
    in_use: Cell<bool>,
    va_args: Option<usize>,
}

impl fmt::Debug for MacroFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out: Vec<_> = self.out.iter().map(|x| *x as char).enumerate().collect();
        write!(
            f,
            "Macro Function: {:?}\n{:?}\nin_use: {:?}",
            out, self.actions, self.in_use
        )
    }
}

impl fmt::Debug for MacroObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out: Vec<_> = self.out.iter().map(|x| *x as char).enumerate().collect();
        write!(f, "Macro Object: {:?}\nin_use: {:?}", out, self.in_use)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Macro {
    Object(MacroObject),
    Function(MacroFunction),
}

#[derive(Clone, Debug)]
pub(crate) enum MacroType<'a> {
    None,
    Object(&'a MacroObject),
    Function((usize, Option<usize>)),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Action {
    Arg(usize),
    Concat(usize),
    Stringify(usize),
    Chunk(usize),
}

impl Action {
    fn is_arg(&self) -> bool {
        match self {
            Action::Arg(_) => true,
            _ => false,
        }
    }

    fn arg_n(&self) -> usize {
        match self {
            Action::Arg(n) => *n,
            _ => 0,
        }
    }
}

impl MacroFunction {
    #[inline(always)]
    pub(crate) fn new(
        out: Vec<u8>,
        actions: Vec<Action>,
        n_args: usize,
        va_args: Option<usize>,
    ) -> Self {
        Self {
            out,
            actions,
            n_args,
            in_use: Cell::new(false),
            va_args,
        }
    }

    #[inline(always)]
    pub(crate) fn eval_parsed_args<'a>(
        &self,
        args: &[Vec<MacroNode<'a>>],
        context: &PContext,
        out: &mut Vec<u8>,
    ) {
        let mut out_pos = 0;
        let mut output = Vec::new();

        for action in self.actions.iter() {
            match action {
                Action::Arg(pos) => {
                    MacroNode::eval_nodes(&args[*pos], context, &mut output);
                }
                Action::Concat(pos) => {
                    MacroNode::make_expr(&args[*pos], &mut output);
                }
                Action::Stringify(pos) => {
                    MacroNode::make_string(&args[*pos], &mut output);
                }
                Action::Chunk(pos) => {
                    output.extend_from_slice(unsafe { &self.out.get_unchecked(out_pos..*pos) });
                    out_pos = *pos;
                }
            }
        }
        output.extend_from_slice(unsafe { &self.out.get_unchecked(out_pos..) });

        let mut lexer = Lexer::new(&output);
        self.in_use.set(true);
        lexer.macro_final_eval(out, context);
        self.in_use.set(false);
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.n_args
    }

    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.n_args == 0
    }
}

impl MacroObject {
    #[inline(always)]
    pub(crate) fn new(out: Vec<u8>, has_id: bool) -> Self {
        Self {
            out,
            has_id,
            in_use: Cell::new(false),
        }
    }

    #[inline(always)]
    pub(crate) fn eval(&self, out: &mut Vec<u8>, context: &PContext) {
        if self.has_id {
            let mut lexer = Lexer::new(&self.out);
            self.in_use.set(true);
            lexer.macro_final_eval(out, context);
            self.in_use.set(false);
        } else {
            out.extend_from_slice(&self.out);
        }
    }
}

impl PContext {
    pub(crate) fn show_if_stack(&self) {
        eprintln!("IF_STACK: {:?}", self.if_stack);
    }

    pub(crate) fn add_if(&mut self, state: IfState) {
        self.if_stack.push(state);
    }

    pub(crate) fn rm_if(&mut self) {
        self.if_stack.pop();
    }

    pub(crate) fn if_state(&self) -> Option<&IfState> {
        self.if_stack.last()
    }

    pub(crate) fn if_change(&mut self, state: IfState) {
        *self.if_stack.last_mut().unwrap() = state;
    }

    pub(crate) fn add_function(&mut self, name: String, mac: MacroFunction) {
        self.macros.insert(name, Macro::Function(mac));
    }

    pub(crate) fn add_object(&mut self, name: String, mac: MacroObject) {
        self.macros.insert(name, Macro::Object(mac));
    }

    pub(crate) fn undef(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub(crate) fn defined(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub(crate) fn eval(&self, name: &str, lexer: &mut Lexer, out: &mut Vec<u8>) -> bool {
        if let Some(mac) = self.get(name) {
            match mac {
                Macro::Object(mac) => {
                    mac.eval(out, &self);
                }
                Macro::Function(mac) => {
                    if let Some(args) = lexer.get_arguments(mac.n_args, mac.va_args.as_ref()) {
                        mac.eval_parsed_args(&args, &self, out);
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Macro> {
        if let Some(mac) = self.macros.get(name) {
            match mac {
                Macro::Object(m) => {
                    return if m.in_use.get() { None } else { Some(mac) };
                }
                Macro::Function(m) => {
                    return if m.in_use.get() { None } else { Some(mac) };
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn get_type(&self, name: &str) -> MacroType {
        if let Some(mac) = self.get(name) {
            match mac {
                Macro::Object(mac) => MacroType::Object(&mac),
                Macro::Function(mac) => MacroType::Function((mac.len(), mac.va_args.clone())),
            }
        } else {
            MacroType::None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lexer::Token;

    macro_rules! eval {
        ( $name: expr, $lexer: expr ) => {{
            let context = $lexer.context.clone();
            let mut res = Vec::new();
            context.eval($name, &mut $lexer, &mut res);
            String::from_utf8(res).unwrap()
        }};
    }

    #[test]
    fn test_macro_in_context() {
        let mut p =
            Lexer::new(b"#define foo x + 1\n#define bar y + x\n#define foobar(x, y) x ## y");
        p.consume_tokens(3);
        assert!(p.context.get("foo").is_some());
        assert!(p.context.get("bar").is_some());
        assert!(p.context.get("foobar").is_some());
    }

    #[test]
    fn test_eval_object() {
        let mut p = Lexer::new(
            concat!(
                "#define foo x + 1\n",
                "#define bar y + /* comment */ x\n",
                "#define oof (x)\n",
                "#define test1 foo\n",
                "#define test2 bar\n",
                "#define test3 oof\n",
            )
            .as_bytes(),
        );
        p.consume_tokens(6);

        assert_eq!(eval!("test1", p), "x + 1");
        assert_eq!(eval!("test2", p), "y + x");
        assert_eq!(eval!("test3", p), "(x)");
    }

    #[test]
    fn test_eval_concat1() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(x, y) x ## y\n",
                "#define bar(x) x\n",
                "#define test1 foo(12, 34)\n",
                "#define test2 foo(12, bar(34))"
            )
            .as_bytes(),
        );
        p.consume_tokens(4);

        assert_eq!(eval!("test1", p), "1234");
        assert_eq!(eval!("test2", p), "12bar(34)");
    }

    #[test]
    fn test_eval_function() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(x) x\n",
                "#define bar(x) x + 1\n",
                "#define test foo(bar(1234))",
            )
            .as_bytes(),
        );
        p.consume_tokens(3);

        assert_eq!(eval!("test", p), "1234 + 1");
    }

    #[test]
    fn test_eval_mix() {
        let mut p = Lexer::new(
            concat!(
                "#define xstr(s) str(s)\n",
                "#define str(s) #s\n",
                "#define foo 4\n",
                "#define test xstr(foo)",
            )
            .as_bytes(),
        );
        p.consume_tokens(4);

        assert_eq!(eval!("test", p), "\"4\"");
    }

    #[test]
    fn test_eval_base() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(a, b) (a) + (b)\n",
                "#define test foo(  123 ,  456  )"
            )
            .as_bytes(),
        );
        p.consume_tokens(2);

        assert_eq!(eval!("test", p), "(123) + (456)");
    }

    #[test]
    fn test_eval_hex() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(x, abc) x + 0x123abc\n",
                "#define test foo(456, 789)"
            )
            .as_bytes(),
        );
        p.consume_tokens(2);

        assert_eq!(eval!("test", p), "456 + 0x123abc");
    }

    #[test]
    fn test_eval_comment() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(a,b,c) a b /* hello world*/     foo c\n",
                "#define bar(a,b,c) a / b // c\n",
                "#define test1 foo(A, B, C)\n",
                "#define test2 bar(A, B, C)"
            )
            .as_bytes(),
        );
        p.consume_tokens(4);

        assert_eq!(eval!("test1", p), "A B foo C");
        assert_eq!(eval!("test2", p), "A / B ");
    }

    #[test]
    fn test_eval_concat2() {
        let mut p = Lexer::new(
            concat!(
                "#define Z(a,b) a ##b\n",
                "#define Y(a,b) c ## d\n",
                "#define X(a,b) c a ## d b\n",
                "#define W(a,b) a c ## b d\n",
                "#define V(a,b) A a##b B\n",
                "#define test1 Z(hello, world)\n",
                "#define test2 Y(hello, world)\n",
                "#define test3 X(hello, world)\n",
                "#define test4 W(hello, world)\n",
                "#define test5 V(hello, world)\n",
            )
            .as_bytes(),
        );
        p.consume_tokens(10);

        assert_eq!(eval!("test1", p), "helloworld");
        assert_eq!(eval!("test2", p), "cd");
        assert_eq!(eval!("test3", p), "c hellod world");
        assert_eq!(eval!("test4", p), "hello cworld d");
        assert_eq!(eval!("test5", p), "A helloworld B");
    }

    #[test]
    fn test_eval_stringify() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(a, b) #a + #b\n",
                "#define bar BAR\n",
                "#define test1 foo(oof, rab)\n",
                "#define test2 foo(bar, bar)"
            )
            .as_bytes(),
        );
        p.consume_tokens(4);

        assert_eq!(eval!("test1", p), "\"oof\" + \"rab\"");
        assert_eq!(eval!("test2", p), "\"bar\" + \"bar\"");
    }

    #[test]
    fn test_eval_stringify_string() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(a) #a\n",
                "#define test foo(R\"delimiter( a string with some \', \" and \n.)delimiter\")"
            )
            .as_bytes(),
        );
        p.consume_tokens(2);

        assert_eq!(
            eval!("test", p),
            "\"R\\\"delimiter( a string with some \\\', \\\" and \\\n.)delimiter\\\"\""
        );
    }

    #[test]
    fn test_eval_auto_ref() {
        let mut p = Lexer::new(concat!("#define foo a foo\n", "#define test foo",).as_bytes());
        p.consume_tokens(2);

        assert_eq!(eval!("test", p), "a foo");
    }

    #[test]
    fn test_eval_auto_ref2() {
        let mut p = Lexer::new(
            concat!(
                "#define FOO x rab bar\n",
                "#define test FOO\n",
                "#define bar FOO\n",
                "#define rab oof\n",
                "#define oof y FOO\n",
            )
            .as_bytes(),
        );

        p.consume_tokens(5);

        assert_eq!(eval!("test", p), "x y FOO FOO");
    }

    #[test]
    fn test_eval_auto_ref3() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(x) x bar(x)\n",
                "#define bar(x) x oof(x)\n",
                "#define oof(x) x foo(x)\n",
                "#define test foo(hello)\n",
            )
            .as_bytes(),
        );

        p.consume_tokens(4);

        assert_eq!(eval!("test", p), "hello hello hello foo(hello)");
    }

    #[test]
    fn test_eval_va() {
        let mut p = Lexer::new(
            concat!(
                "#define foo(x, y, ...) x y __VA_ARGS__\n",
                "#define bar(x, y, toto...) x y toto\n",
                "#define foo1(toto...) printf(toto)\n",
                "#define test1 foo(a, b, c, d, e, f)\n",
                "#define test2 bar(a, b, c, d, e, f)\n",
                "#define test3 foo1(a, b)\n",
                "#define test4 foo1()\n",
            )
            .as_bytes(),
        );

        p.consume_tokens(7);

        assert_eq!(eval!("test1", p), "a b c,d,e,f");
        assert_eq!(eval!("test2", p), "a b c,d,e,f");
        assert_eq!(eval!("test3", p), "printf(a,b)");
        assert_eq!(eval!("test4", p), "printf()");
    }
}
