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
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::string;

#[macro_use]
extern crate lazy_static;

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
                for a in env::args() {
                    if (a.to_lowercase().ends_with(".fmd")) {
                        out.push(a);
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
struct FMD<'a> {
    _tokens: Vec<&'a str>,
    _incres: INCLUDED_RESOURCES,
}

impl<'a> FMD<'a> {
    fn new() -> FMD<'a> {
        FMD::<'a> {
            _tokens: Vec::new(),
            _incres: INCLUDED_RESOURCES::new(),
        }
    }

    fn pre_tokenize(self, text: &'a str) -> FMD<'a> {
        if (text.is_empty()) {
            return FMD::<'a> {
                _tokens: self._tokens,
                _incres: self._incres,
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
        }
    }
    // ! TODO: Replace \n with <br> or something
    fn replace_ibus(self) -> FMD<'a> {
        if (self._tokens.is_empty()) {
            return FMD::<'a> {
                _tokens: self._tokens,
                _incres: self._incres,
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
            } else {
                out_str = t;
            }
            out.push(out_str);
        }

        FMD::<'a> {
            _tokens: out,
            _incres: self._incres,
        }
    }

    pub fn concat_tokens(self) -> String {
        let mut str: String = String::new();
        for t in self._tokens {
            str.push_str(t);
        }
        str
    }
}

pub struct INCLUDED_RESOURCES {
    pub pyscript: bool,
}

impl INCLUDED_RESOURCES {
    fn new() -> INCLUDED_RESOURCES {
        INCLUDED_RESOURCES { pyscript: false }
    }
}
static HEADER_PYSCRIPT: &str = r##"<link rel="stylesheet" href="https://pyscript.net/latest/pyscript.css"/>        
<script defer src="https://pyscript.net/latest/pyscript.js"></script>"##;

static HTML_HEADER: &str = r##"<!DOCTYPE html>
<html>
    <head>                
        <link rel="stylesheet" href="style.css" />
    </head>
    <body>
        <form class="color-picker" action="">
            <fieldset>
                <legend class="visually-hidden">Pick a color scheme</legend>
                <label for="light" class="visually-hidden">Light</label>
                <input type="radio" name="theme" id="light" checked>

                <label for="pink" class="visually-hidden">Pink theme</label>
                <input type="radio" id="pink" name="theme">

                <label for="blue" class="visually-hidden">Blue theme</label>
                <input type="radio" id="blue" name="theme">

                <label for="green" class="visually-hidden">Green theme</label>
                <input type="radio" id="green" name="theme">

                <label for="dark" class="visually-hidden">Dark theme</label>
                <input type="radio" id="dark" name="theme">
            </fieldset>
        </form>
        <main>
            <div class="wrapper">
                <p>"##;
static HTML_FOOTER: &str = r##"</p>
            </div>
        </main>
        <script>
            const colorThemes = document.querySelectorAll('[name="theme"]');

            // store theme
            const storeTheme = function (theme) {
            localStorage.setItem("theme", theme);
            };

            // set theme when visitor returns
            const setTheme = function () {
            const activeTheme = localStorage.getItem("theme");
            colorThemes.forEach((themeOption) => {
                if (themeOption.id === activeTheme) {
                themeOption.checked = true;
                }
            });
            // fallback for no :has() support
            document.documentElement.className = activeTheme;
            };

            colorThemes.forEach((themeOption) => {
            themeOption.addEventListener("click", () => {
                storeTheme(themeOption.id);
                // fallback for no :has() support
                document.documentElement.className = themeOption.id;
            });
            });

            document.onload = setTheme();
        </script>
    </body>
</html>"##;
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
    //let test = "L[sub]o[/sub]o[sup]k[/sup]i[sub]n[/sub]g [u]for[/u] a [b][i][color=blue]quick[/color][/i][/b] [color =\"#FF0000\"]brown[/color] fox [s]that[/s] jumps over a[b][color=pink]lazy [/color]dog[/b] Find out more [url=localhost]here![/url] or [url=localhost]there![/url]. Lorem Ipsum Salts.";

    let args = CommandLineArguments::new();

    //TODO: If args != contain fmd_files || path -> Process * in working dir
    //TODO: If args contains path -> Process * in path
    for f in args.fmd_files {
        let html_filename = &f[0..f.len() - 4];
        let contents = fs::read_to_string(f.clone())
            .expect(format!("Unable to read or find file: {}", f).as_str());
        let fmd = FMD::new();

        write_html(
            html_filename,
            format!(
                "{}{}{}",
                HTML_HEADER,
                fmd.pre_tokenize(contents.as_str())
                    .replace_ibus()
                    .concat_tokens(),
                HTML_FOOTER
            )
            .as_str(),
        );
    }

    fs::copy("./src/style.css", "style.css").unwrap();
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
