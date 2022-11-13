use tower_lsp::lsp_types::{Position, TextDocumentContentChangeEvent};
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

pub struct ParsedCode {
    pub parser: Parser,
    pub code: String,
    pub tree: Tree,
}

impl ParsedCode {
    pub fn new(code: String) -> Self {
        let mut parser = Parser::new();
        extern "C" {
            fn tree_sitter_fvf() -> Language;
        }
        let language = unsafe { tree_sitter_fvf() };
        parser
            .set_language(language)
            .expect("Error loading fvf grammar");
        let tree = parser.parse(&code, None).unwrap();
        Self { parser, code, tree }
    }

    pub fn edit(&mut self, events: &[TextDocumentContentChangeEvent]) {
        for event in events {
            let range = event.range.unwrap();
            let start_ofs = find_offset(&self.code, range.start).unwrap();
            let end_ofs = find_offset(&self.code, range.end).unwrap();
            self.code.replace_range(start_ofs..end_ofs, &event.text);

            let new_end_position = match event.text.rfind('\n') {
                Some(ind) => {
                    let num_newlines = event.text.bytes().filter(|&c| c == b'\n').count();
                    Point {
                        row: range.start.line as usize + num_newlines,
                        column: event.text.len() - ind,
                    }
                }
                None => Point {
                    row: range.end.line as usize,
                    column: range.end.character as usize + event.text.len(),
                },
            };

            self.tree.edit(&InputEdit {
                start_byte: start_ofs,
                old_end_byte: end_ofs,
                new_end_byte: start_ofs + event.text.len(),
                start_position: to_point(range.start),
                old_end_position: to_point(range.end),
                new_end_position,
            });
        }
        self.tree = self.parser.parse(&self.code, Some(&self.tree)).unwrap();
    }
}

fn find_offset(text: &str, pos: Position) -> Option<usize> {
    let mut line_start = 0;
    for _ in 0..pos.line {
        line_start = text[line_start..].find('\n')? + line_start + 1;
    }
    Some(line_start + pos.character as usize)
}

fn to_point(p: Position) -> Point {
    Point {
        row: p.line as usize,
        column: p.character as usize,
    }
}

// fn to_position(p: Point) -> Position {
//     Position {
//         line: p.row as u32,
//         character: p.column as u32,
//     }
// }
//
// fn node_text<'a>(code: &'a str, node: &Node) -> &'a str {
//     &code[node.start_byte()..node.end_byte()]
// }
