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
[title][/title]         // Order Table of Contents by Title, unless filename starts with chN_
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
    title_open,
    title_close,
    code_open,
    code_close,
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
        m
    };
}

#[derive(Clone)]
pub struct FMD {
    _tokens: Vec<String>,
    _incres: INCLUDED_RESOURCES,
    _definitions: Vec<DEFINITION>,
    _filename: String,
    _title: String,
}

impl FMD {
    pub fn new() -> Self {
        Self {
            _tokens: Vec::new(),
            _incres: INCLUDED_RESOURCES::new(),
            _definitions: Vec::new(),
            _filename: String::new(),
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

        let mut out: Vec<String> = Vec::new();
        let mut code_block = false;
        let mut code_block_language = String::new();

        for t in self._tokens {
            let mut out_str = String::new();
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
            } else if (REGEX_HASHMAP[&REGEX_NAME::newline].is_match(&t)) {
                out_str = "<br>".to_string();
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
            } else {
                out_str = t;
                out_str = out_str
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
                    .replace("®", "&reg;");

                if (code_block) {
                    out_str = FMD::format_code_block(out_str, &code_block_language);
                }
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
            _ => {}
        }
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

    pub fn get_filename(&self) -> String {
        self._filename.to_owned()
    }
    pub fn set_filename(self, filename: impl Into<String>) -> Self {
        let mut str: String = filename.into();
        if (str.len() == 0) {
            return Self {
                _tokens: self._tokens,
                _incres: self._incres,
                _definitions: self._definitions,
                _filename: self._filename,
                _title: self._title,
            };
        }

        if (str.len() > 4) {
            if (str[str.len() - 4..str.len()].eq_ignore_ascii_case(".fmd")) {
                str.pop();
                str.pop();
                str.pop();
                str.pop();
            }
        }
        Self {
            _tokens: self._tokens,
            _incres: self._incres,
            _definitions: self._definitions,
            _filename: str,
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

    pub fn addJob(self, filename: impl Into<String>) -> JOBS {
        let mut job = self.jobs;
        let mut fmd = FMD::new();
        fmd = fmd.set_filename(filename.into());
        job.push(fmd);
        JOBS {
            state: self.state,
            jobs_total: self.jobs_total + 1,
            jobs_remaining: self.jobs_remaining + 1,
            jobs: job,
        }
    }
}

pub fn generate_toc(toc_titles: Vec<(String, String)>) -> String {
    let mut out: String = r##"<nav class="table-of-contents">
    <ol>"##
        .to_owned();

    for (t, f) in toc_titles {
        out.push_str(r##"<li class="toc-content">"##);
        out.push_str(format!(r##"<a href="{}.html">"##, &f[0..f.len()]).as_str());
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
