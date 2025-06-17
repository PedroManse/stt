//use std::collections::HashMap;
use colored::Colorize;
use std::ops::Range;

fn parse_line(line: &str) -> (&str, Range<usize>) {
    let (path, range) = line.split_once(':').unwrap();
    let (start, end) = range.split_once("..").unwrap();
    let start = start.parse().unwrap();
    let end = end.parse().unwrap();
    (path, start..end)
}

fn span_expand_safe(span: Range<usize>, max: usize) -> Range<usize> {
    span.start.saturating_sub(10)..(span.end + 10).min(max)
}

fn main() {
    let reqs = std::env::args().skip(1);
    //let mut files: HashMap<String, String> = HashMap::new();
    for req in reqs {
        let (path, span) = parse_line(&req);
        let file_cont = std::fs::read_to_string(path).unwrap();
        let span = span_expand_safe(span, file_cont.len());
        println!("===[ {} ]===", path.bright_magenta());
        println!("{}", &file_cont[span]);
    }
}
