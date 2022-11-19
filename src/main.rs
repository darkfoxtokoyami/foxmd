/* TODO
[i][/i]
[b][/b]
[u][/u]
[s][/s]
[color=<name/#NNNNNN>][/color]
[sup][/sup]
[sub][/sub]

[citation=""][/citation]
[definition=""][/definition]
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
[chapter][/chapter]     // Makes this the root node for a directory. I.e. This file will be the chapter page, the inner text will be the title you see on the Table of Contents,
                       .//  Everything else in the same directory will be the leaf nodes of the chapter. 1 [chapter] per directory
                       .// Probably defunct. easier to do something like- If a directory exists with the same name as the fmd, make the fmd a chapter
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

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#[cfg(test)]
mod tests;
use regex::Regex;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use serde_json::value::Index;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::string;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[macro_use]
extern crate lazy_static;

static MAIN_JS: &str = include_str!("main.js");
static STYLE_CSS: &str = include_str!("style.css");

struct CommandLineArguments {
    _args: Vec<String>,
    fmd_files: Vec<String>,
}

impl CommandLineArguments {
    fn new() -> CommandLineArguments {
        CommandLineArguments {
            _args: env::args().collect(),
            fmd_files: {
                let mut out = Vec::new();
                let mut found_fmd_or_path = false;
                for a in env::args() {
                    if (a.to_lowercase().ends_with(".fmd")) {
                        found_fmd_or_path = true;
                        out.push(a);
                    }
                }
                //TODO Detect if path is in args
                if (!found_fmd_or_path) {
                    for f in fs::read_dir("./").unwrap() {
                        let s = f.unwrap().path().display().to_string().to_lowercase();
                        if (s.ends_with(".fmd")) {
                            out.push(s);
                        }
                    }
                }
                out
            },
        }
    }

    fn getFMDFiles(self, args: Vec<String>) -> Vec<String> {
        let mut out = Vec::new();
        for a in env::args() {
            if (a.to_lowercase().ends_with(".fmd")) {
                out.push(a);
            }
        }

        out
    }
}

#[derive(Clone)]
pub struct INCLUDED_RESOURCES {
    pub pyscript: bool,
}

impl INCLUDED_RESOURCES {
    fn new() -> INCLUDED_RESOURCES {
        INCLUDED_RESOURCES { pyscript: false }
    }
}
const HEADER_PYSCRIPT: &str = include_str!("html/pyscript_header.html");
const HTML_HEADER: &str = include_str!("html/header.html");
const HTML_FOOTER: &str = include_str!("html/footer.html");

#[derive(Serialize, Deserialize, Debug)]
struct ASTNode {
    t: String,
    val: String,
    child: Option<Box<ASTNode>>,
}

#[derive(Clone)]
struct DEFINITION {
    word: String,
    text: String,
}

impl DEFINITION {
    fn new() -> DEFINITION {
        DEFINITION {
            word: String::new(),
            text: String::new(),
        }
    }

    fn is_empty(&self) -> bool {
        (self.word.is_empty())
    }
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
    newline,
    definition_open,
    definition_close,
}

lazy_static! {
    static ref REGEX_HASHMAP: HashMap<REGEX_NAME, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            REGEX_NAME::italics_open,
            Regex::new(r"(?i)\[\s*i\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::italics_close,
            Regex::new(r"(?i)\[\s*/\s*i\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::bold_open,
            Regex::new(r"(?i)\[\s*b\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::bold_close,
            Regex::new(r"(?i)\[\s*/\s*b\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::underline_open,
            Regex::new(r"(?i)\[\s*u\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::underline_close,
            Regex::new(r"(?i)\[\s*/\s*u\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::strikethrough_open,
            Regex::new(r"(?i)\[\s*s\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::strikethrough_close,
            Regex::new(r"(?i)\[\s*/\s*s\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::superscript_open,
            Regex::new(r"(?i)\[\s*sup\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::superscript_close,
            Regex::new(r"(?i)\[\s*/\s*sup\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::subscript_open,
            Regex::new(r"(?i)\[\s*sub\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::subscript_close,
            Regex::new(r"(?i)\[\s*/\s*sub\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::color_open,
            Regex::new(r##"(?i)\[\s*color\s*=\s*"??((\w+)|(#(\d|\w){6}))"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::color_close,
            Regex::new(r"(?i)\[\s*/\s*color\s*]").unwrap(),
        );
        m.insert(REGEX_NAME::newline, Regex::new(r"\n").unwrap());
        m.insert(
            REGEX_NAME::definition_open,
            Regex::new(r##"(?i)\[\s*definition\s*=\s*"??(\w+)"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::definition_close,
            Regex::new(r"(?i)\[\s*/\s*definition\s*]").unwrap(),
        );
        m
    };
}

#[derive(Clone)]
struct FMD<'a> {
    _tokens: Vec<&'a str>,
    _incres: INCLUDED_RESOURCES,
    _definitions: Vec<DEFINITION>,
}

impl<'a> FMD<'a> {
    fn new() -> FMD<'a> {
        FMD::<'a> {
            _tokens: Vec::new(),
            _incres: INCLUDED_RESOURCES::new(),
            _definitions: Vec::new(),
        }
    }

    // Breaks a string up into tokens, separated by [] tags
    fn pre_tokenize(self, text: &'a str) -> FMD<'a> {
        if (text.is_empty()) {
            return FMD::<'a> {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
            };
        }

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
        FMD::<'a> {
            _tokens: out,
            _incres: self._incres,
            _definitions: self._definitions,
        }
    }

    //
    fn parse_definitions(mut self) -> FMD<'a> {
        if (self._tokens.is_empty()) {
            return FMD::<'a> {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
            };
        }

        let mut out: Vec<DEFINITION> = Vec::new();
        let mut found_definition_open = false;
        let mut found_definition_open_idx = 0;

        for i in 0..self._tokens.len() {
            if (REGEX_HASHMAP[&REGEX_NAME::definition_open].is_match(self._tokens[i])) {
                found_definition_open = true; // ! Warning: This does NOT check or support nested definitions! Not sure if I should panic at, or support those
                found_definition_open_idx = i;
            } else if (REGEX_HASHMAP[&REGEX_NAME::definition_close].is_match(self._tokens[i])) {
                if (found_definition_open) {
                    self._definitions.push(DEFINITION::new());
                    // get definition word
                    let idx = self._definitions.len() - 1;
                    self._definitions[idx].word = self._tokens[found_definition_open_idx]
                        [Regex::new(r##"\[\s*definition\s*=\s*"?"##)
                            .unwrap()
                            .find(self._tokens[found_definition_open_idx])
                            .unwrap()
                            .end()
                            ..Regex::new(r##""?]"##)
                                .unwrap()
                                .find(self._tokens[found_definition_open_idx])
                                .unwrap()
                                .start()]
                        .to_string();

                    // get text
                    let mut text = String::new();
                    for j in found_definition_open_idx + 1..i + 1 {
                        text.push_str(self._tokens[j]);
                    }
                    let fmd = FMD::new();
                    let def_text = fmd
                        .pre_tokenize(text.as_str())
                        .replace_ibus()
                        .concat_tokens();
                    self._definitions[idx].text = def_text;
                }

                found_definition_open = false;
            }
        }

        FMD::<'a> {
            _tokens: self._tokens,
            _incres: self._incres,
            _definitions: self._definitions,
        }
    }

    // TODO: Copy this function or something; and make it work with definitions. Is there a way to do that without just copy+pasting my for loop in rust? It's kind of finnicky about that stuff
    // Replaces basic [] tags (e.g. italics, bold, underline, strikethrough) with corresponding html tags
    fn replace_ibus(self) -> FMD<'a> {
        if (self._tokens.is_empty()) {
            return FMD::<'a> {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
            };
        }

        let mut out: Vec<&str> = Vec::new();

        for t in self._tokens {
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
            } else if (REGEX_HASHMAP[&REGEX_NAME::newline].is_match(t)) {
                out_str = "<br>";
            } else if (REGEX_HASHMAP[&REGEX_NAME::definition_open].is_match(t)) {
                out.push(r#"<span class="definition_word"><b>"#);
                out.push(
                    &t[Regex::new(r##"\[\s*definition\s*=\s*"?"##)
                        .unwrap()
                        .find(t)
                        .unwrap()
                        .end()
                        ..Regex::new(r##""?]"##).unwrap().find(t).unwrap().start()],
                );
                out_str = r#":  </b></span><span class="definition_text">"#;
                // out_str = "<span style = \"color:red;\">";
            } else if (REGEX_HASHMAP[&REGEX_NAME::definition_close].is_match(t)) {
                out_str = "</span>";
            } else {
                out_str = t;
            }
            out.push(out_str);
        }

        FMD::<'a> {
            _tokens: out,
            _incres: self._incres,
            _definitions: self._definitions,
        }
    }

    pub fn concat_tokens(&self) -> String {
        let mut str: String = String::new();
        for i in 0..self._tokens.len() {
            str.push_str(self._tokens[i]);
        }

        let mut out = String::new();
        out.push_str("<BR>DEFINITIONS<BR>");
        for i in 0..self._definitions.len() {
            out.push_str(self._definitions[i].word.as_str());
            out.push_str(": ");
            out.push_str(self._definitions[i].text.as_str());
            out.push_str("<br>");
        }

        str.push_str(out.as_str());
        str
    }

    pub fn get_definitions(&self) -> Vec<DEFINITION> {
        let mut out_def = Vec::new();
        for i in 0..self._definitions.len() {
            out_def.push(DEFINITION {
                word: self._definitions[i].word.to_owned(),
                text: self._definitions[i].text.to_owned(),
            });
        }
        out_def
    }
}

fn main() {
    // let test = Document("Hello, world!".to_string());
    //let test = "L[sub]o[/sub]o[sup]k[/sup]i[sub]n[/sub]g [u]for[/u] a [b][i][color=blue]quick[/color][/i][/b] [color =\"#FF0000\"]brown[/color] fox [s]that[/s] jumps over a[b][color=pink]lazy [/color]dog[/b] Find out more [url=localhost]here![/url] or [url=localhost]there![/url]. Lorem Ipsum Salts.";

    let args = CommandLineArguments::new();
    // fs::copy("./src/style.css", "style.css").expect("./src/style.css not found!");
    write_style_css();

    //TODO: If args != contain fmd_files || path -> Process * in working dir
    //TODO: If args contains path -> Process * in path
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut definitions: Arc<Mutex<Vec<DEFINITION>>> = Arc::new(Mutex::new(Vec::new()));

    for f in args.fmd_files {
        let def = Arc::clone(&definitions);
        handles.push(thread::spawn(move || {
            let html_filename = &f[0..f.len() - 4];
            let contents = fs::read_to_string(f.clone())
                .expect(format!("Unable to read or find file: {}", f).as_str());
            let mut fmd = FMD::new();

            fmd = fmd
                .pre_tokenize(contents.as_str())
                .parse_definitions()
                .replace_ibus();
            write_html(
                html_filename,
                format!("{}{}{}", HTML_HEADER, fmd.concat_tokens(), HTML_FOOTER).as_str(),
            );
            {
                let mut t_def = def.lock().unwrap();
                let mut tt_def = fmd.get_definitions();
                tt_def.append(&mut *t_def);
                *t_def = tt_def.to_owned();
            }
        }));
    }

    // Wait for all threads to finish
    for h in handles {
        h.join().unwrap();
    }

    let appendix_defs = &*definitions.lock().unwrap();
    if (appendix_defs.len() > 0) {
        let mut out = "DEFINITIONS: <br>".to_string();
        for d in appendix_defs {
            out.push_str(d.word.as_str());
            out.push_str(": ");
            out.push_str(d.text.as_str());
            out.push_str("<br>");
        }

        write_html(
            "appendix_a.html",
            format!("{}{}{}", HTML_HEADER, out, HTML_FOOTER).as_str(),
        );
    }
}

pub fn write_style_css() {
    let f = format!("style.css");
    let path = Path::new(&f);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("Unable to create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(STYLE_CSS.as_bytes()) {
        Err(why) => panic!("Unable to write {}: {}", display, why),
        Ok(_) => println!("Successfully wrote {}", display),
    }
}

pub fn write_html(file_name: &str, html: &str) {
    let f = format!("{}.html", file_name);
    let path = Path::new(&f);
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
