/*
Done:
[i][/i]
[b][/b]
[u][/u]
[s][/s]
[color=<name/#NNNNNN>][/color]
[sup][/sup]
[sub][/sub]
[url=""][/url]
[code=LANGUAGE][/code]
[noparse][/noparse]
[img][/img]

 :TODO:
[citation=""][/citation]
[definition=""][/definition]
[video=""][/video]
[spoiler][/spoiler]
[title][/title]         // Order Table of Contents by Title, unless filename starts with chN_
[titlesub][/titlesub]
[tableofcontents][/tableofcontents]
[h=N][/h]
[python="<python program>"]<alternate/subscript text>[/python]
[rust="<rust program>"]<alternate/subscript text>[/rust]
[csv=][/csv]
[excel][/excel]
[table][/table]
[math][/math]
[latex][/latex]
[gutter][/gutter]       // Basically, to put definitions of things next to paragraphs, in the righthand margins of the page I guess? I dunno
[chapter][/chapter]     // Makes this the root node for a directory. I.e. This file will be the chapter page, the inner text will be the title you see on the Table of Contents,
                       .//  Everything else in the same directory will be the leaf nodes of the chapter. 1 [chapter] per directory
                       .// Probably defunct. easier to do something like- If a directory exists with the same name as the fmd, make the fmd a chapter
[profile][/profile]     // Picture on left, text on right
[ig][/ig]               // Image Grid.  Displays everything in the img directory in a grid

Filename:
    chN_                // If the filename starts with ch(Number)_ then order by filename rather than title. Or maybe I should just sort by filename to begin with?
Special Note: I'm aware that this is probably closer to BB Code than MarkDown, but.. The alternative would be for me to call Fox's MarkDown as Fox's BBC. Soo...
*/
// TODO Title(done) -> Table of Contents -> Multi-Pass Definitions
// TODO Sanitize Output for HTML compatibility (E.g. < -> &lt;)
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
use threadpool::ThreadPool;
use walkdir::WalkDir;

#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate threadpool;

pub const MAIN_JS: &str = include_str!("main.js");
pub const STYLE_CSS: &str = include_str!("style.css");
pub const HEADER_PYSCRIPT: &str = include_str!("html/pyscript_header.html");
pub const HTML_HEADER: &str = include_str!("html/header.html");
pub const HTML_FOOTER: &str = include_str!("html/footer.html");

pub struct CommandLineArguments {
    _args: Vec<String>,
    pub fmd_files: Vec<String>,
}

impl CommandLineArguments {
    pub fn new() -> CommandLineArguments {
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

    pub fn getFMDFiles(self, args: Vec<String>) -> Vec<String> {
        let mut out = Vec::new();
        for a in env::args() {
            if (a.to_lowercase().ends_with(".fmd")) {
                out.push(a);
            }
        }

        out
    }
}
pub struct FMD_FILES_AND_TITLES {
    pub title: String,
    pub filename: String,
}

impl FMD_FILES_AND_TITLES {
    pub fn new(mut self) {
        self.title = String::new();
        self.filename = String::new();
    }
}
#[derive(Clone)]
pub struct INCLUDED_RESOURCES {
    pub pyscript: bool,
}

impl INCLUDED_RESOURCES {
    pub fn new() -> INCLUDED_RESOURCES {
        INCLUDED_RESOURCES { pyscript: false }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ASTNode {
    t: String,
    val: String,
    child: Option<Box<ASTNode>>,
}

pub enum CodeFormatting {
    html,
    css,
    javascript,
    python,
    rust,
    cpp,
    cs,
    java,
    go,
    gb,
    x86,
    arm,
    fortran,
    cobol,
    fs,
    vb,
}
#[derive(Clone)]
pub struct DEFINITION {
    pub word: String,
    pub text: String,
}

impl DEFINITION {
    pub fn new() -> DEFINITION {
        DEFINITION {
            word: String::new(),
            text: String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
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
    bold_close,
    bold_open,
    code_close,
    code_open,
    color_close,
    color_open,
    definition_close,
    definition_open,
    img_close,
    img_open,
    italics_close,
    italics_open,
    newline,
    noparse_close,
    noparse_open,
    strikethrough_close,
    strikethrough_open,
    subscript_close,
    subscript_open,
    superscript_close,
    superscript_open,
    title_close,
    title_open,
    underline_close,
    underline_open,
    url_close,
    url_open,
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
        // m.insert(REGEX_NAME::newline, Regex::new(r"\n").unwrap());
        m.insert(
            REGEX_NAME::definition_open,
            Regex::new(r##"(?i)\[\s*definition\s*=\s*"??(\w+)"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::definition_close,
            Regex::new(r"(?i)\[\s*/\s*definition\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::title_open,
            Regex::new(r"(?i)\[\s*title\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::title_close,
            Regex::new(r"(?i)\[\s*/\s*title\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::code_open,
            Regex::new(r##"(?i)\[\s*code\s*=\s*"??(\w+)"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::code_close,
            Regex::new(r"(?i)\[\s*/\s*code\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::url_open,
            Regex::new(r##"(?i)\[\s*url\s*=\s*"??(\S+?)"??]"##).unwrap(),
        );
        m.insert(
            REGEX_NAME::url_close,
            Regex::new(r"(?i)\[\s*/\s*url\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::noparse_open,
            Regex::new(r"(?i)\[\s*noparse\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::noparse_close,
            Regex::new(r"(?i)\[\s*/\s*noparse\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::img_open,
            Regex::new(r"(?i)\[\s*img\s*]").unwrap(),
        );
        m.insert(
            REGEX_NAME::img_close,
            Regex::new(r"(?i)\[\s*/\s*img\s*]").unwrap(),
        );
        m
    };
}

#[derive(Clone)]
pub struct FMD {
    _tokens: Vec<String>,
    _incres: INCLUDED_RESOURCES,
    _definitions: Vec<DEFINITION>,
    _filename: Option<walkdir::DirEntry>,
    _title: String,
}

impl FMD {
    pub fn new() -> Self {
        Self {
            _tokens: Vec::new(),
            _incres: INCLUDED_RESOURCES::new(),
            _definitions: Vec::new(),
            _filename: None,
            _title: String::new(),
        }
    }

    // Breaks a string up into tokens, separated by [] tags
    pub fn pre_tokenize(self, _text: impl Into<String>) -> Self {
        let text: String = _text.into();
        if (text.is_empty()) {
            return Self {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
                _filename: self._filename,
                _title: self._title,
            };
        }

        let mut out: Vec<String> = Vec::new();
        let mut bounds: Vec<MDBounds> = Vec::new();

        // for r in regs {
        for k in REGEX_HASHMAP.keys() {
            for m in REGEX_HASHMAP[k].find_iter(&text) {
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
                out.push(text[index..b.start].to_owned());
            }
            out.push(text[b.start..b.end].to_owned());
            index = b.end;
        }
        if (index != text.len()) {
            out.push(text[index..text.len()].to_owned());
        }
        Self {
            _tokens: out,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: self._filename,
            _title: self._title,
        }
    }

    //
    pub fn parse_definitions(mut self) -> Self {
        if (self._tokens.is_empty()) {
            return Self {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
                _filename: self._filename,
                _title: self._title,
            };
        }

        let mut out: Vec<DEFINITION> = Vec::new();
        let mut found_definition_open = false;
        let mut found_definition_open_idx = 0;

        for i in 0..self._tokens.len() {
            if (REGEX_HASHMAP[&REGEX_NAME::definition_open].is_match(&self._tokens[i])) {
                found_definition_open = true; // ! Warning: This does NOT check or support nested definitions! Not sure if I should panic at, or support those
                found_definition_open_idx = i;
            } else if (REGEX_HASHMAP[&REGEX_NAME::definition_close].is_match(&self._tokens[i])) {
                if (found_definition_open) {
                    self._definitions.push(DEFINITION::new());
                    // get definition word
                    let idx = self._definitions.len() - 1;
                    self._definitions[idx].word = self._tokens[found_definition_open_idx]
                        [Regex::new(r##"\[\s*definition\s*=\s*"?"##)
                            .unwrap()
                            .find(&self._tokens[found_definition_open_idx])
                            .unwrap()
                            .end()
                            ..Regex::new(r##""?]"##)
                                .unwrap()
                                .find(&self._tokens[found_definition_open_idx])
                                .unwrap()
                                .start()]
                        .to_string();

                    // get text
                    let mut text = String::new();
                    for j in found_definition_open_idx + 1..i + 1 {
                        text.push_str(&self._tokens[j]);
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

        Self {
            _tokens: self._tokens,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: self._filename,
            _title: self._title,
        }
    }

    pub fn parse_title(self) -> Self {
        if (self._tokens.is_empty()) {
            return Self {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
                _filename: self._filename,
                _title: self._title,
            };
        }

        let mut found_title_open = false;
        let mut found_title_open_idx = 0;
        let mut title = String::new();

        for i in 0..self._tokens.len() {
            if (REGEX_HASHMAP[&REGEX_NAME::title_open].is_match(&self._tokens[i])) {
                found_title_open = true; // ! Warning: This does NOT check or support nested titles!
                found_title_open_idx = i;
            } else if (REGEX_HASHMAP[&REGEX_NAME::title_close].is_match(&self._tokens[i])) {
                if (found_title_open) {
                    // get title
                    for j in found_title_open_idx + 1..i {
                        title.push_str(&self._tokens[j]);
                    }
                }

                found_title_open = false;
            }
        }

        Self {
            _tokens: self._tokens,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: self._filename,
            _title: title,
        }
    }

    // TODO: Copy this function or something; and make it work with definitions. Is there a way to do that without just copy+pasting my for loop in rust? It's kind of finnicky about that stuff
    // Replaces basic [] tags (e.g. italics, bold, underline, strikethrough) with corresponding html tags
    pub fn replace_ibus(self) -> Self {
        if (self._tokens.is_empty()) {
            return Self {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
                _filename: self._filename,
                _title: self._title,
            };
        }

        fn sanitize_string(str: impl Into<String>) -> String {
            str.into()
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace(r##"""##, "&quot;")
                .replace("'", "&apos;")
                .replace("¢", "&cent;")
                .replace("£", "&pound;")
                .replace("¥", "&yen;")
                .replace("€", "&euro;")
                .replace("©", "&copy;")
                .replace("®", "&reg;")
                .replace("\t", "&nbsp;&nbsp;&nbsp;&nbsp;")
                .replace("\r\n", "<br>")
                .replace("\n", "<br>")
        }

        let mut out: Vec<String> = Vec::new();
        let mut code_block = false;
        let mut code_block_language = String::new();
        let mut noparse = false;

        for t in self._tokens {
            let mut out_str = String::new();
            if (!noparse) {
                if (REGEX_HASHMAP[&REGEX_NAME::italics_open].is_match(&t)) {
                    out_str = "<i>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::italics_close].is_match(&t)) {
                    out_str = "</i>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::bold_open].is_match(&t)) {
                    out_str = "<b>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::bold_close].is_match(&t)) {
                    out_str = "</b>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::underline_open].is_match(&t)) {
                    out_str = "<u>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::underline_close].is_match(&t)) {
                    out_str = "</u>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::strikethrough_open].is_match(&t)) {
                    out_str = "<span style = \"text-decoration:line-through;\">".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::strikethrough_close].is_match(&t)) {
                    out_str = "</span>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::superscript_open].is_match(&t)) {
                    out_str = "<sup>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::superscript_close].is_match(&t)) {
                    out_str = "</sup>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::subscript_open].is_match(&t)) {
                    out_str = "<sub>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::subscript_close].is_match(&t)) {
                    out_str = "</sub>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::color_open].is_match(&t)) {
                    out.push(r#"<span style = "color:"#.to_owned());
                    out.push(
                        t[Regex::new(r##"\[\s*color\s*=\s*"?"##)
                            .unwrap()
                            .find(&t)
                            .unwrap()
                            .end()
                            ..Regex::new(r##""?]"##).unwrap().find(&t).unwrap().start()]
                            .to_owned(),
                    );
                    out_str = r#";">"#.to_string();
                    // out_str = "<span style = \"color:red;\">";
                } else if (REGEX_HASHMAP[&REGEX_NAME::color_close].is_match(&t)) {
                    out_str = "</span>".to_string();
                // } else if (REGEX_HASHMAP[&REGEX_NAME::newline].is_match(&t)) {
                //     out_str = "<br>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::definition_open].is_match(&t)) {
                    out.push(r#"<span class="definition-word"><b>"#.to_owned());
                    out.push(
                        t[Regex::new(r##"\[\s*definition\s*=\s*"?"##)
                            .unwrap()
                            .find(&t)
                            .unwrap()
                            .end()
                            ..Regex::new(r##""?]"##).unwrap().find(&t).unwrap().start()]
                            .to_owned(),
                    );
                    out_str = r#":  </b></span><span class="definition-text">"#.to_string();
                    // out_str = "<span style = \"color:red;\">";
                } else if (REGEX_HASHMAP[&REGEX_NAME::definition_close].is_match(&t)) {
                    out_str = "</span>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::title_open].is_match(&t)) {
                    out_str = r#"<span class="title">"#.to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::title_close].is_match(&t)) {
                    out_str = "</span>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::code_open].is_match(&t)) {
                    code_block_language = t[Regex::new(r##"\[\s*code\s*=\s*"?"##)
                        .unwrap()
                        .find(&t)
                        .unwrap()
                        .end()
                        ..Regex::new(r##""?]"##).unwrap().find(&t).unwrap().start()]
                        .to_owned();
                    out_str = r#"<code>"#.to_string();
                    code_block = true;
                } else if (REGEX_HASHMAP[&REGEX_NAME::code_close].is_match(&t)) {
                    out_str = "</code>".to_string();
                    code_block = false;
                } else if (REGEX_HASHMAP[&REGEX_NAME::url_open].is_match(&t)) {
                    out.push(r#"<a href=""#.to_owned());
                    out.push(
                        t[Regex::new(r##"\[\s*url\s*=\s*"?"##)
                            .unwrap()
                            .find(&t)
                            .unwrap()
                            .end()
                            ..Regex::new(r##""?]"##).unwrap().find(&t).unwrap().start()]
                            .to_owned(),
                    );
                    out_str = r#"" target="_blank" rel="noopener noreferrer">"#.to_string();
                    // out_str = "<a style = \"color:red;\">";
                } else if (REGEX_HASHMAP[&REGEX_NAME::url_close].is_match(&t)) {
                    out_str = "</a>".to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::noparse_open].is_match(&t)) {
                    noparse = true;
                } else if (REGEX_HASHMAP[&REGEX_NAME::img_open].is_match(&t)) {
                    out_str = r#"<img src=""#.to_string();
                } else if (REGEX_HASHMAP[&REGEX_NAME::img_close].is_match(&t)) {
                    out_str = r##"" style="max-width: 100%;"></img>"##.to_string();
                } else {
                    out_str = sanitize_string(t);

                    if (code_block) {
                        out_str = out_str.replace(" ", "&nbsp;");
                        out_str = FMD::format_code_block(out_str, &code_block_language);
                    }
                }
            } else if (REGEX_HASHMAP[&REGEX_NAME::noparse_close].is_match(&t)) {
                noparse = false;
            } else {
                out_str = sanitize_string(t);
            }
            out.push(out_str);
        }

        Self {
            _tokens: out,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: self._filename,
            _title: self._title,
        }
    }

    pub fn format_code_block(code: String, language: impl Into<String>) -> String {
        let lang = String::from(language.into());
        let mut out = String::new();
        match lang.to_lowercase().as_str() {
            "html" => {
                out = code.replace(
                    "&lt;html&gt;",
                    r##"<span class="code-html-tag">&lt;html&gt;</span>"##,
                )
            }
            _ => out = code,
        }
        let mut str = r##"<div class="code-block" style="height: 400px; width: 100%; background: lightgrey; overflow-x: scroll; overflow-y: scroll;"><div class="code-line-number-margin unselectable" style="top: 0px; left: 0px; width: 5%; float: left; font-size: 0.75em; white-space: nowrap;">"##.to_string();
        println!("Find <br>? {}", out.find("<br>").unwrap().to_string());

        while (out.starts_with("<br>")) {
            out = out[4..out.len()].to_string();
        }
        let br_count = out.matches("<br>").count() + 1;
        for n in 1..br_count {
            str.push_str(format!("<div>{}</div>", n.to_string()).as_str());
        }
        str.push_str(r##"</div><div class="code-block-code" style="background: lightgrey; width: 95%; float: left; font-size: 0.75em;white-space: nowrap;">"##);
        str.push_str(&out);
        out = str;
        out.push_str(r##"</div><div style="clear: both;"></div></div>"##);
        out
    }

    pub fn concat_tokens(&self) -> String {
        let mut str: String = String::new();
        for i in 0..self._tokens.len() {
            str.push_str(&self._tokens[i]);
        }

        // let mut out = String::new();
        // out.push_str("<BR>DEFINITIONS<BR>");
        // for i in 0..self._definitions.len() {
        //     out.push_str(self._definitions[i].word.as_str());
        //     out.push_str(": ");
        //     out.push_str(self._definitions[i].text.as_str());
        //     out.push_str("<br>");
        // }

        // str.push_str(out.as_str());
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

    pub fn get_file(&self) -> String {
        let l = self
            ._filename
            .clone()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .len();

        self._filename.clone().unwrap().path().to_str().unwrap()[0..l - 4].to_owned()
    }
    pub fn get_filename(&self) -> String {
        let l = self
            ._filename
            .clone()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .len();
        self._filename
            .clone()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()[0..l - 4]
            .to_owned()
    }
    pub fn set_filename(self, filename: walkdir::DirEntry) -> Self {
        // Assumes that filename is a valid DirEntry. Should be using get_fmd_files to get the DirEntries
        Self {
            _tokens: self._tokens,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: Some(filename),
            _title: self._title,
        }
    }

    pub fn get_title(&self) -> String {
        self._title.to_owned()
    }

    pub fn get_tokens(&self) -> Vec<String> {
        self._tokens.clone()
    }
}

pub enum JOB_STATE {
    S0_Init,
    S1_Parsing,
    S2_ResolveDef,
    S3_BuildTOC,
}
pub struct JOBS {
    state: JOB_STATE,
    jobs_total: usize,
    jobs_remaining: usize,
    jobs: Vec<FMD>,
}

impl JOBS {
    pub fn new() -> JOBS {
        JOBS {
            state: JOB_STATE::S0_Init,
            jobs_total: 0,
            jobs_remaining: 0,
            jobs: Vec::new(),
        }
    }

    pub fn addJob(self, filename: walkdir::DirEntry) -> JOBS {
        let mut job = self.jobs;
        let mut fmd = FMD::new();
        fmd = fmd.set_filename(filename);
        job.push(fmd);
        JOBS {
            state: self.state,
            jobs_total: self.jobs_total + 1,
            jobs_remaining: self.jobs_remaining + 1,
            jobs: job,
        }
    }
}

pub fn generate_toc(mut toc_titles: Vec<(String, String)>) -> String {
    // toc_titles Vec<(title, filename)>
    let mut out: String = r##"<nav class="table-of-contents">
    <ol>"##
        .to_owned();
    for (t, f) in toc_titles {
        out.push_str(r##"<li class="toc-content">"##);
        out.push_str(format!(r##"<a href="{}.html">"##, &f).as_str());
        if (t.is_empty()) {
            out.push_str(&f[0..f.len()]);
        } else {
            out.push_str(t.as_str());
        }

        out.push_str(r##"</a></li>"##);
    }
    out.push_str(
        r##"    </ol>
    </nav>"##,
    );
    out
}

pub fn print_dirs() {
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| match e.file_type().is_dir() {
            true => Some(e),
            false => None,
        })
    {
        println!("{}", entry.path().display());
    }
}

pub fn print_fmds() {
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| match e.file_type().is_dir() {
            true => None,
            false => Some(e),
        })
        .filter_map(|e| {
            if (e.file_name().len() > 4) {
                match &e.file_name().to_str()?[e.file_name().len() - 4..e.file_name().len()] {
                    ".fmd" => Some(e),
                    _ => None,
                }
            } else {
                None
            }
        })
    {
        println!("{}", entry.file_name().to_str().unwrap());
    }
}

pub fn get_fmd_files() -> Vec<walkdir::DirEntry> {
    let mut out = Vec::new();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| match e.file_type().is_dir() {
            true => None,
            false => Some(e),
        })
        .filter_map(|e| {
            if (e.file_name().len() > 4) {
                match &e.file_name().to_str()?[e.file_name().len() - 4..e.file_name().len()] {
                    ".fmd" => Some(e),
                    _ => None,
                }
            } else {
                None
            }
        })
    {
        println!("Found: {}", &entry.path().display());
        out.push(entry);
    }
    out
}

pub fn parse_fmds_in_dir_recursively(directory: impl Into<String>) {
    let dir = directory.into();
    let args = CommandLineArguments::new();
    let args2 = CommandLineArguments::new();
    // fs::copy("./src/style.css", "style.css").expect("./src/style.css not found!");
    write_style_css();

    //TODO: If args != contain fmd_files || path -> Process * in working dir
    //TODO: If args contains path -> Process * in path
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut definitions: Arc<Mutex<Vec<DEFINITION>>> = Arc::new(Mutex::new(Vec::new()));
    let mut fmds: Arc<Mutex<Vec<FMD>>> = Arc::new(Mutex::new(Vec::new()));
    let mut jobs: JOBS = JOBS::new();
    let n_workers = 4;
    let thread_pool = ThreadPool::new(num_cpus::get());
    println!("File Types:");
    print_fmds();
    println!("Building Docs with {} threads", num_cpus::get());

    // Need to figure out a way to deal withmultiple passes.
    // Job State = Open/Parsing, Definition Resolution, Building Table of Contents, Completed
    // Jobs_Total  -> Amount of files to process, make sure this is >0
    // Jobs_Remaining -> Don't move to next state until this hits zero
    for f in get_fmd_files() {
        //jobs = jobs.addJob(f.to_owned());
        let def = Arc::clone(&definitions);
        let t_fmds = Arc::clone(&fmds);
        thread_pool.execute(move || {
            let mut fmd = FMD::new();
            fmd = fmd.set_filename(f.clone());
            let html_filename = fmd.get_filename();
            let contents = fs::read_to_string(&f.path())
                .expect(format!("Unable to read or find file: {}", &f.path().display()).as_str());

            fmd = fmd
                .pre_tokenize(contents.as_str())
                .parse_definitions()
                .parse_title()
                .replace_ibus();

            // Push Definitions
            {
                let mut t_def = def.lock().unwrap();
                let mut tt_def = fmd.get_definitions();
                tt_def.append(&mut *t_def);
                *t_def = tt_def.to_owned();
            }

            // Push FMDs
            {
                let mut tt_fmds = t_fmds.lock().unwrap();
                let mut ttt_fmds: Vec<FMD> = Vec::new();
                ttt_fmds.append(&mut tt_fmds);
                ttt_fmds.push(fmd);
                *tt_fmds = ttt_fmds.to_owned();
            }
        });
    }

    // Wait for all threads to finish
    thread_pool.join();

    // Get Titles for Table of Contents
    let mut toc_titles: Vec<(String, String)> = Vec::new();
    {
        let t_fmds = fmds.lock().unwrap();
        for fmd in &*t_fmds {
            println!("Found: {}", fmd.get_file());
            toc_titles.push((fmd.get_title(), fmd.get_file()));
        }
    }
    // Sort the Table of Contents by Filename.  This allowes us to [title] a file however we want, but
    //      still control the ordering of the chapters in the Table of Contents. E.g. CH1_MyFile.fmd with [title]B[/title]
    //      will come before CH2_MyFile.fmd with [title]A[/title]
    toc_titles.sort_by(|a, b| a.1.cmp(&b.1));

    // Compile Table of Contents and write Definitions' Appendix to disk
    // WARNING: definitions are locked here, but the lock doesn't go out of scope until the end of main!
    let appendix_defs = &*definitions.lock().unwrap();
    if (appendix_defs.len() > 0) {
        toc_titles.push(("Appendix A".to_owned(), "appendix_a".to_owned()));
    }

    let table_of_contents = generate_toc(toc_titles);

    //println!("table_of_contents:\n{}", &table_of_contents);

    if (appendix_defs.len() > 0) {
        let mut out = "DEFINITIONS: <br>".to_string();
        for d in appendix_defs {
            out.push_str(r#"<span class="definition-word"><b>"#);
            out.push_str(d.word.as_str());
            out.push_str(": ");
            out.push_str(r#":  </b></span><span class="definition-text">"#);
            out.push_str(d.text.as_str());
            out.push_str("</span><br>");
        }

        write_html(
            "appendix_a",
            format!(
                "{}{}{}{}",
                HTML_HEADER, &table_of_contents, out, HTML_FOOTER
            )
            .as_str(),
        );
    }

    // Write parsed fmds to file.
    let i_fmds = Arc::clone(&fmds);
    {
        // I know this is janky, but the speculation is that CPU cycles and RAM R/W is cheaper than writing to disk sequentially (as opposed to in parallel like below).
        //      Ideally there should be a performance benefit when writing 100's of files for bigger docs.
        let t_fmds = i_fmds.lock().unwrap().to_vec();

        for f in t_fmds {
            //jobs = jobs.addJob(f.to_owned());
            let toc = table_of_contents.to_owned();
            //println!("toc:\n{}", &toc);
            thread_pool.execute(move || {
                write_html(
                    &f.get_file(),
                    format!("{}{}{}{}", HTML_HEADER, toc, f.concat_tokens(), HTML_FOOTER).as_str(),
                );
            });
        }
    }
    // Wait for all threads to finish
    thread_pool.join();
}
fn write_style_css() {
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

fn write_html(file_name: &str, html: &str) {
    let f = format!("{}.html", file_name);
    let dir_depth = get_dir_depth(file_name);
    let path = Path::new(&f);
    let display = path.display();
    let mut html_out = html.to_owned();
    println!("Dir Depth: {}", &dir_depth.to_string());
    if (dir_depth > 1) {
        for _ in 1..dir_depth {
            html_out = html_out.replace("style.css", "../style.css");
            html_out = html_out.replace(
                r##"<li class="toc-content"><a href=""##,
                r##"<li class="toc-content"><a href="../"##,
            );
        }
    }
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("Unable to create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `html` string to `file`, returns `io::Result<()>`
    match file.write_all(html_out.as_bytes()) {
        Err(why) => panic!("Unable to write {}: {}", display, why),
        Ok(_) => println!("Successfully wrote {}", display),
    }
}

fn get_dir_depth(file_name: impl Into<String>) -> usize {
    let f = file_name.into();
    let out = if (f.matches("/").count() > f.matches(r##"\"##).count()) {
        f.matches("/").count()
    } else {
        f.matches(r##"\"##).count()
    };
    out
}
