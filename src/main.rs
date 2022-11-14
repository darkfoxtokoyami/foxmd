/* TODO
[i][/i]
[b][/b]
[u][/u]
[s][/s]
[color=<name/#NNNNNN>][/color]
[sup][/sup]
[sub][/sub]

[citation=""][/citation]

[img=""][/img]
[video=""][/video]
[url=""][/url]
[code=LANGUAGE][/code]
[spoiler][/spoiler]
[noparse][/noparse]
[title][/title]
[titlesub][/titlesub]
[tableofcontents][/tableofcontents]
[h=N][/h]
[python="<python program>"]<alternate/subscript text>[/python]
[rust="<rust program>"]<alternate/subscript text>[/rust]
[csv=""][/csv]
[excel=""][/excel]
[math][/math]
[latex][/latex]
[gutter][/gutter]       // Basically, to put definitions of things next to paragraphs, in the righthand margins of the page I guess? I dunno
*/

/*

Document
    : StatementList
    ;

Statement
    : Text
    | MarkDownCode
    ;

Text
    : String
    ;

MarkDownCode
    : [Identifier=Argument]Text[/Identifier]
    ;
*/
#[cfg(test)]
mod tests;
use regex::Regex;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use serde_json::value::Index;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::string;
#[macro_use]
extern crate lazy_static;
static HTML_HEADER: &str = r##"<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" type="text/css" href="style.css">
        <link rel="stylesheet" href="https://pyscript.net/latest/pyscript.css" />
        <script defer src="https://pyscript.net/latest/pyscript.js"></script>
    </head>
    <body>
        <p>"##;
static HTML_FOOTER: &str = "</p>
    </body>
</html>";
#[derive(Serialize, Deserialize, Debug)]
struct ASTNode {
    t: String,
    val: String,
    child: Option<Box<ASTNode>>,
}
#[derive(Clone, Copy)]
pub struct MDBounds {
    start: usize,
    end: usize,
}

#[derive(PartialEq, Eq, Hash)]
enum REGEX_NAME {
    italics_open,
    italics_close,
    bold_open,
    bold_close,
    underline_open,
    underline_close,
    strikethrough_open,
    strikethrough_close,
    superscript_open,
    superscript_close,
    subscript_open,
    subscript_close,
    color_open,
    color_close,
}

lazy_static! {
    static ref REGEX_HASHMAP: HashMap<REGEX_NAME, Regex> = {
        let mut m = HashMap::new();
        m.insert(REGEX_NAME::italics_open, Regex::new(r"\[\s*i\s*]").unwrap());
        m.insert(
            REGEX_NAME::italics_close,
            Regex::new(r"\[\s*/\s*i\s*]").unwrap(),
        );
        m.insert(REGEX_NAME::bold_open, Regex::new(r"\[\s*b\s*]").unwrap());
        m.insert(
            REGEX_NAME::bold_close,
            Regex::new(r"\[\s*/\s*b\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::underline_open,
            Regex::new(r"\[\s*u\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::underline_close,
            Regex::new(r"\[\s*/\s*u\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::strikethrough_open,
            Regex::new(r"\[\s*s\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::strikethrough_close,
            Regex::new(r"\[\s*/\s*s\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::superscript_open,
            Regex::new(r"\[\s*sup\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::superscript_close,
            Regex::new(r"\[\s*/\s*sup\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::subscript_open,
            Regex::new(r"\[\s*sub\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::subscript_close,
            Regex::new(r"\[\s*/\s*sub\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::color_open,
            Regex::new(r##"\[\s*color\s*=\s*"??((\w+)|(#(\d|\w){6}))"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::color_close,
            Regex::new(r"\[\s*/\s*color\s*]").unwrap(),
        );
        m
    };
}
fn main() {
    // let test = Document("Hello, world!".to_string());
    //! TODO Change this before making it public, and wipe the old git repo //
    let test = "L[sub]o[/sub]o[sup]k[/sup]i[sub]n[/sub]g [u]for[/u] a [b][i][color=blue]quick[/color][/i][/b] [color =\"#FF0000\"]brown[/color] fox [s]that[/s] jumps over a[b][color=pink]lazy [/color]dog[/b] Find out more [url=localhost]here![/url] or [url=localhost]there![/url]. Lorem Ipsum Salts.";

    let t = pre_tokenize(test);
    let u = replace_ibus(t);
    let v = concat_tokens(u);
    // for x in t {
    //     println!("{}", x);
    // }

    write_html(format!("{}{}{}", HTML_HEADER, v, HTML_FOOTER).as_str());
}

pub fn write_html(html: &str) {
    let path = Path::new("index.html");
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("Unable to create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `html` string to `file`, returns `io::Result<()>`
    match file.write_all(html.as_bytes()) {
        Err(why) => panic!("Unable to write {}: {}", display, why),
        Ok(_) => println!("Successfully wrote {}", display),
    }
}

pub fn pre_tokenize(text: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let mut bounds: Vec<MDBounds> = Vec::new();

    // for r in regs {
    for k in REGEX_HASHMAP.keys() {
        for m in REGEX_HASHMAP[k].find_iter(text) {
            let t: MDBounds = MDBounds {
                start: m.start(),
                end: m.end(),
            };
            bounds.push(t);
            //println!("{:?}, ", m.start().to_string());
        }
    }

    // Regex matches are found out of order, so they need to be reorganized
    bounds.sort_by(|a, b| a.start.cmp(&b.start));

    let mut index: usize = 0;
    for b in bounds {
        if (index != b.start) {
            out.push(&text[index..b.start]);
        }
        out.push(&text[b.start..b.end]);
        index = b.end;
    }
    if (index != text.len()) {
        out.push(&text[index..text.len()]);
    }
    return out;
}

pub fn replace_ibus(tokens: Vec<&str>) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();

    for t in tokens {
        let mut out_str = "";
        if (REGEX_HASHMAP[&REGEX_NAME::italics_open].is_match(t)) {
            out_str = "<i>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::italics_close].is_match(t)) {
            out_str = "</i>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::bold_open].is_match(t)) {
            out_str = "<b>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::bold_close].is_match(t)) {
            out_str = "</b>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::underline_open].is_match(t)) {
            out_str = "<u>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::underline_close].is_match(t)) {
            out_str = "</u>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::strikethrough_open].is_match(t)) {
            out_str = "<span style = \"text-decoration:line-through;\">";
        } else if (REGEX_HASHMAP[&REGEX_NAME::strikethrough_close].is_match(t)) {
            out_str = "</span>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::superscript_open].is_match(t)) {
            out_str = "<sup>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::superscript_close].is_match(t)) {
            out_str = "</sup>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::subscript_open].is_match(t)) {
            out_str = "<sub>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::subscript_close].is_match(t)) {
            out_str = "</sub>";
        } else if (REGEX_HASHMAP[&REGEX_NAME::color_open].is_match(t)) {
            out.push(r#"<span style = "color:"#);
            out.push(
                &t[Regex::new(r##"\[\s*color\s*=\s*"?"##)
                    .unwrap()
                    .find(t)
                    .unwrap()
                    .end()
                    ..Regex::new(r##""?]"##).unwrap().find(t).unwrap().start()],
            );
            out_str = r#";">"#;
            // out_str = "<span style = \"color:red;\">";
        } else if (REGEX_HASHMAP[&REGEX_NAME::color_close].is_match(t)) {
            out_str = "</span>";
        } else {
            out_str = t;
        }
        out.push(out_str);
    }
    out
}

pub fn concat_tokens(tokens: Vec<&str>) -> String {
    let mut str: String = String::new();
    for t in tokens {
        str.push_str(t);
    }
    str
}
fn Document(document: String) -> ASTNode {
    ASTNode {
        t: "Document".to_string(),
        val: "".to_string(),
        child: Some(Box::new(Text(document))),
    }
}

fn Text(document_val: String) -> ASTNode {
    ASTNode {
        t: "Text".to_string(),
        val: document_val.to_owned(),
        child: None,
    }
}
