use std::{
    fs::File,
    io::{BufReader, Read},
    mem,
    path::Path,
};

use utf8_chars::BufReadCharsExt;

use crate::warlocs::{Locs, Warlocs};

pub struct Visitor<T: Read> {
    reader: BufReader<T>,
    context: VisitorContext,
    stats: Warlocs,
    lookahead: Option<char>,
    curr_string: String,
    curr_line_no: usize,
    debug: bool,
}

#[derive(Debug, Copy, Clone)]
enum VisitorContext {
    Main,
    Tests,
    Example,
}

#[derive(Default, Debug, Copy, Clone)]
struct LineContext {
    has_code: bool,
    has_comment_start: bool,
    has_doc_comment_start: bool,
}

impl LineContext {
    fn is_inside_comment(&self) -> bool {
        self.has_comment_start || self.has_doc_comment_start
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    LineBreak,
    WhiteSpace,
    TestBlockStart,
    CodeBlockOpen,
    CodeBlockClose,
    CommentStart,
    DocCommentStart,
    CommentBlockOpen,
    CommentBlockClose,
    DocComentBlockOpen,
    EndOfStatement,
    DoubleBackSlash,
    DoubleQuote,
    EscapedDoubleQuote,
    StringBlockOpen,
    StringBlockClose,
    DoubleStringBlockOpen,
    DoubleStringBlockClose,
    Other,
}

impl VisitorContext {
    fn from_file_path(path: impl AsRef<Path>) -> Self {
        for component in path.as_ref().components() {
            match component {
                std::path::Component::Normal(os_str)
                    if os_str == "tests" || os_str == "tests.rs" =>
                {
                    return Self::Tests;
                }
                std::path::Component::Normal(os_str) if os_str == "examples" => {
                    return Self::Example;
                }
                _ => {}
            }
        }

        Self::Main
    }
}

impl Visitor<File> {
    pub fn new(file_path: impl AsRef<Path>, debug: bool) -> Self {
        let file = File::open(&file_path).unwrap_or_else(|e| {
            panic!(
                "failed to read file {}: {e}",
                file_path.as_ref().to_str().unwrap_or_default()
            )
        });
        let mut reader = BufReader::new(file);
        let context = VisitorContext::from_file_path(file_path);

        let lookahead = reader.chars().next().and_then(|c| c.ok());

        Self {
            reader,
            context,
            stats: Warlocs::default(),
            lookahead,
            curr_string: String::new(),
            curr_line_no: 1,
            debug,
        }
    }
}

impl<T: Read> Visitor<T> {
    pub fn visit_file(mut self) -> Warlocs {
        self.stats.file_count += 1;

        self.visit_code(self.context);

        self.stats
    }

    fn visit_code(&mut self, context: VisitorContext) {
        let line_context = LineContext::default();
        self.visit_code_block(context, line_context, true);
    }

    fn visit_test_block(&mut self) {
        self.skip_line(
            VisitorContext::Tests,
            LineContext {
                has_code: true,
                ..Default::default()
            },
        );

        let mut line_context = LineContext::default();

        while let Some(token) = self.next_token() {
            match token {
                Token::LineBreak => {
                    self.finish_line(VisitorContext::Tests, line_context);
                    line_context = LineContext::default();
                }
                Token::EndOfStatement => {
                    line_context.has_code = true;
                    self.skip_line(VisitorContext::Tests, line_context);
                    return;
                }
                Token::CodeBlockOpen => {
                    self.visit_code_block(VisitorContext::Tests, line_context, false);
                    line_context.has_code = true;
                    self.skip_line(VisitorContext::Tests, line_context);
                    return;
                }
                Token::WhiteSpace => {}
                _ => {
                    if !line_context.is_inside_comment() {
                        line_context.has_code = true;
                    }
                }
            }
        }
    }

    fn skip_line(&mut self, context: VisitorContext, line_context: LineContext) {
        while let Some(char) = self.next_char() {
            if char == '\n' {
                break;
            }
        }

        self.finish_line(context, line_context);
    }

    fn visit_code_block(
        &mut self,
        context: VisitorContext,
        line_context: LineContext,
        till_the_end: bool,
    ) {
        let mut line_context = line_context;
        while let Some(token) = self.next_token() {
            match token {
                Token::LineBreak => {
                    self.finish_line(context, line_context);
                    line_context = LineContext::default();
                }
                Token::WhiteSpace => {}
                Token::CommentStart => {
                    line_context.has_comment_start = true;
                    self.skip_line(context, line_context);
                    line_context = LineContext::default();
                }
                Token::DocCommentStart => {
                    line_context.has_doc_comment_start = true;
                    self.skip_line(context, line_context);
                    line_context = LineContext::default();
                }
                Token::CommentBlockOpen => {
                    self.visit_comment_block(context, false);
                    line_context.has_comment_start = true;
                }
                Token::DocComentBlockOpen => {
                    self.visit_comment_block(context, true);
                    line_context.has_doc_comment_start = true;
                }
                Token::TestBlockStart => {
                    self.visit_test_block();
                }
                Token::CodeBlockOpen => {
                    self.visit_code_block(context, line_context, false);
                    line_context.has_code = true;
                }
                Token::CodeBlockClose => {
                    if !till_the_end {
                        return;
                    }
                }
                Token::DoubleQuote => {
                    self.visit_string(context);
                    line_context.has_code = true;
                }
                Token::StringBlockOpen => {
                    self.visit_string_block(context, Token::StringBlockClose);
                    line_context.has_code = true;
                }
                Token::DoubleStringBlockOpen => {
                    self.visit_string_block(context, Token::DoubleStringBlockClose);
                    line_context.has_code = true;
                }
                _ => line_context.has_code = true,
            }
        }
    }

    fn visit_string_block(&mut self, context: VisitorContext, closing_token: Token) {
        let mut line_context = LineContext {
            has_code: true,
            has_comment_start: false,
            has_doc_comment_start: false,
        };

        while let Some(token) = self.next_token() {
            match token {
                Token::LineBreak => {
                    self.finish_line(context, line_context);
                    line_context = LineContext::default();
                }
                v if v == closing_token => {
                    return;
                }
                _ => line_context.has_code = true,
            }
        }
    }

    fn visit_string(&mut self, context: VisitorContext) {
        let mut line_context = LineContext {
            has_code: true,
            has_comment_start: false,
            has_doc_comment_start: false,
        };

        while let Some(token) = self.next_token() {
            match token {
                Token::LineBreak => {
                    self.finish_line(context, line_context);
                    line_context = LineContext::default();
                }
                Token::DoubleQuote => return,
                _ => line_context.has_code = true,
            }
        }
    }

    fn visit_comment_block(&mut self, context: VisitorContext, is_doc: bool) {
        let mut line_context = LineContext {
            has_code: false,
            has_comment_start: !is_doc,
            has_doc_comment_start: is_doc,
        };

        while let Some(token) = self.next_token() {
            match token {
                Token::LineBreak => {
                    self.finish_line(context, line_context);
                    line_context = LineContext::default();
                }
                Token::CommentBlockOpen => {
                    self.visit_comment_block(context, false);
                }
                Token::CommentBlockClose => {
                    return;
                }
                Token::DocComentBlockOpen => {
                    self.visit_comment_block(context, true);
                }
                Token::WhiteSpace => {}
                _ => {
                    line_context.has_comment_start = !is_doc;
                    line_context.has_doc_comment_start = is_doc;
                }
            }
        }
    }

    fn finish_line(&mut self, context: VisitorContext, line_context: LineContext) {
        let curr = std::mem::take(&mut self.curr_string);
        let line = self.curr_line_no;
        self.curr_line_no += 1;

        let stats = self.mut_stats(context);

        if line_context.has_code {
            stats.code += 1;

            if self.debug {
                eprint!("{line}: CODE: {curr}");
            }
        } else if line_context.has_doc_comment_start {
            stats.docs += 1;
            if self.debug {
                eprint!("{line}: DOCS: {curr}");
            }
        } else if line_context.has_comment_start {
            stats.comments += 1;
            if self.debug {
                eprint!("{line}: COMM: {curr}");
            }
        } else {
            stats.whitespaces += 1;
            if self.debug {
                eprint!("{line}: WHITE: {curr}");
            }
        }
    }

    fn mut_stats(&mut self, context: VisitorContext) -> &mut Locs {
        match context {
            VisitorContext::Main => &mut self.stats.main,
            VisitorContext::Tests => &mut self.stats.tests,
            VisitorContext::Example => &mut self.stats.examples,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        let next_char = self.next_char()?;
        let token = match next_char {
            '\n' => Token::LineBreak,
            '/' if self.lookahead == Some('/') => {
                let _ = self.next_char();
                if self.lookahead == Some('/') || self.lookahead == Some('!') {
                    let next_char = self.next_char()?;
                    if next_char == '/' && self.lookahead == Some('/') {
                        Token::CommentStart
                    } else {
                        Token::DocCommentStart
                    }
                } else {
                    Token::CommentStart
                }
            }
            '/' if self.lookahead == Some('*') => {
                let mut string = '/'.to_string();
                self.collect_while(&mut string, |c| c == '!' || c == '*' || c == '/');
                match string.as_str() {
                    "/**" | "/*!" => Token::DocComentBlockOpen,
                    v if v.ends_with("*/") => Token::WhiteSpace,
                    _ => Token::CommentBlockOpen,
                }
            }
            '*' if self.lookahead == Some('/') => {
                let _ = self.next_char();
                Token::CommentBlockClose
            }
            '#' if self.lookahead == Some('[') => {
                let mut string = '#'.to_string();
                self.collect_while(&mut string, |c| c != ']' && c != '\n');

                if let Some(next) = self.lookahead {
                    match next {
                        ']' => {
                            let _ = self.next_char();
                            string.push(next)
                        }
                        _ => return Some(Token::Other),
                    }
                }

                match string.as_str() {
                    "#[cfg(test)]" | "#[test]" => Token::TestBlockStart,
                    _ => Token::Other,
                }
            }
            '{' => Token::CodeBlockOpen,
            '}' => Token::CodeBlockClose,
            ';' => Token::EndOfStatement,
            '\\' if self.lookahead == Some('\\') => {
                let _ = self.next_char();
                Token::DoubleBackSlash
            }
            '\\' if self.lookahead == Some('"') => {
                let _ = self.next_char();
                Token::EscapedDoubleQuote
            }
            '"' if self.lookahead == Some('#') => {
                let mut string = '"'.to_string();
                self.collect_while(&mut string, |c| c == '#');
                match string.as_ref() {
                    "\"#" => Token::StringBlockClose,
                    "\"##" => Token::DoubleStringBlockClose,
                    _ => Token::Other,
                }
            }
            '"' => Token::DoubleQuote,
            'r' if self.lookahead == Some('#') => {
                let mut string = 'r'.to_string();
                self.collect_while(&mut string, |c| c == '#' || c == '"');
                match string.as_ref() {
                    "r#\"" => Token::StringBlockOpen,
                    "r##\"" => Token::DoubleStringBlockOpen,
                    _ => Token::Other,
                }
            }
            v if v.is_whitespace() => Token::WhiteSpace,
            _ => Token::Other,
        };

        Some(token)
    }

    fn collect_while(&mut self, string: &mut String, mut predicate: impl FnMut(char) -> bool) {
        while let Some(next_char) = self.lookahead {
            if predicate(next_char) {
                let _ = self.next_char();
                string.push(next_char);
            } else {
                break;
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        use utf8_chars::BufReadCharsExt;

        let c = mem::replace(
            &mut self.lookahead,
            self.reader.chars().next().and_then(|c| c.ok()),
        );

        if let Some(c) = c {
            self.curr_string.push(c);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use utf8_chars::BufReadCharsExt;

    use super::*;

    fn stats(file: &str) -> Warlocs {
        let mut reader = BufReader::new(file.as_bytes());
        let lookahead = reader.chars().next().and_then(|c| c.ok());

        Visitor {
            reader,
            context: VisitorContext::Main,
            stats: Warlocs::default(),
            lookahead,
            curr_string: String::new(),
            curr_line_no: 1,
            debug: true,
        }
        .visit_file()
    }

    #[test]
    fn empty_file() {
        let file = "\n";
        let stats = stats(file);

        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.main.whitespaces, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn one_empty_string() {
        let file = "  \t\t \n";
        let stats = stats(file);

        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.main.whitespaces, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn one_code_string() {
        let file = "mod lib;\n";
        let stats = stats(file);

        assert_eq!(stats.main.code, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn single_comment() {
        let file = "   // Comment\n";
        let stats = stats(file);

        assert_eq!(stats.main.comments, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn single_doc() {
        let file = "   /// Documentation\n";
        let stats = stats(file);

        assert_eq!(stats.main.docs, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn single_module_doc() {
        let file = "   //! Documentation\n";
        let stats = stats(file);

        assert_eq!(stats.main.docs, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn comment_block() {
        let file = "   /* comment */ \n";
        let stats = stats(file);

        assert_eq!(stats.main.comments, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn multiline_comment_block() {
        let file = r#"   /* 

        comment 
        */ 
"#;

        let stats = stats(file);

        assert_eq!(stats.main.comments, 3);
        assert_eq!(stats.main.whitespaces, 1);
        assert_eq!(stats.main.sum(), 4);
    }

    #[test]
    fn doc_comment_block() {
        let file = "   /** comment */ \n";
        let stats = stats(file);

        assert_eq!(stats.main.docs, 1);
        assert_eq!(stats.main.sum(), 1);
    }

    #[test]
    fn multiline_doc_comment_block() {
        let file = r#"   /*!

        comment 
        */ 
"#;

        let stats = stats(file);

        assert_eq!(stats.main.docs, 3);
        assert_eq!(stats.main.whitespaces, 1);
        assert_eq!(stats.main.sum(), 4);
    }

    #[test]
    fn comment_in_string_literals() {
        let file = r#"
let string = "Not a comment /*";
let a = 1;
"#;

        let stats = stats(file);

        assert_eq!(stats.main.comments, 0);
        assert_eq!(stats.main.code, 2);
    }

    #[test]
    fn test_block() {
        let file = r#"
#[cfg(test)]
mod tests {

    use super::*;

}
"#;

        let stats = stats(file);

        assert_eq!(stats.tests.code, 4);
        assert_eq!(stats.tests.whitespaces, 2);
        assert_eq!(stats.tests.sum(), 6);
    }

    #[test]
    fn multiline_string_literals() {
        let file = r##"
let string = r#"

This is a string
// This is also a string

"#;

"##;

        let stats = stats(file);

        assert_eq!(stats.main.code, 4);
        assert_eq!(stats.main.whitespaces, 4);
        assert_eq!(stats.main.sum(), 8);
    }
}
