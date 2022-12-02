#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#[cfg(test)]
mod tests;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use foxmd::*;
use threadpool::ThreadPool;

fn main() {
    // let test = Document("Hello, world!".to_string());
    //let test = "L[sub]o[/sub]o[sup]k[/sup]i[sub]n[/sub]g [u]for[/u] a [b][i][color=blue]quick[/color][/i][/b] [color =\"#FF0000\"]brown[/color] fox [s]that[/s] jumps over a[b][color=pink]lazy [/color]dog[/b] Find out more [url=localhost]here![/url] or [url=localhost]there![/url]. Lorem Ipsum Salts.";

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
    println!("Building Docs with {} threads", num_cpus::get());
    // Need to figure out a way to deal withmultiple passes.
    // Job State = Open/Parsing, Definition Resolution, Building Table of Contents, Completed
    // Jobs_Total  -> Amount of files to process, make sure this is >0
    // Jobs_Remaining -> Don't move to next state until this hits zero
    for f in args.fmd_files {
        //jobs = jobs.addJob(f.to_owned());
        let def = Arc::clone(&definitions);
        let t_fmds = Arc::clone(&fmds);
        thread_pool.execute(move || {
            let html_filename = &f[0..f.len() - 4];
            let contents = fs::read_to_string(f.clone())
                .expect(format!("Unable to read or find file: {}", f).as_str());
            let mut fmd = FMD::new();
            fmd = fmd.set_filename(&f);
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
            println!("Found: {}", fmd.get_filename());
            toc_titles.push((fmd.get_title(), fmd.get_filename()));
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
                    &f.get_filename()[0..f.get_filename().len()],
                    format!("{}{}{}{}", HTML_HEADER, toc, f.concat_tokens(), HTML_FOOTER).as_str(),
                );
            });
        }
    }
    // Wait for all threads to finish
    thread_pool.join();
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

// fn Document(document: String) -> ASTNode {
//     ASTNode {
//         t: "Document".to_string(),
//         val: "".to_string(),
//         child: Some(Box::new(Text(document))),
//     }
// }

// fn Text(document_val: String) -> ASTNode {
//     ASTNode {
//         t: "Text".to_string(),
//         val: document_val.to_owned(),
//         child: None,
//     }
// }
