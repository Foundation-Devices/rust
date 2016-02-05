#![feature(plugin)]
#![plugin(clippy)]

#![allow(unused)]
#![deny(invalid_regex, trivial_regex)]

extern crate regex;

use regex::Regex;

const OPENING_PAREN : &'static str = "(";
const NOT_A_REAL_REGEX : &'static str = "foobar";

fn syntax_error() {
    let pipe_in_wrong_position = Regex::new("|");
    //~^ERROR: regex syntax error: empty alternate
    let wrong_char_ranice = Regex::new("[z-a]");
    //~^ERROR: regex syntax error: invalid character class range

    let some_regex = Regex::new(OPENING_PAREN);
    //~^ERROR: regex syntax error on position 0: unclosed

    let closing_paren = ")";
    let not_linted = Regex::new(closing_paren);
}

fn trivial_regex() {
    let trivial_eq = Regex::new("^foobar$");
    //~^ERROR: trivial regex
    //~|HELP consider using `==` on `str`s

    let trivial_starts_with = Regex::new("^foobar");
    //~^ERROR: trivial regex
    //~|HELP consider using `str::starts_with`

    let trivial_ends_with = Regex::new("foobar$");
    //~^ERROR: trivial regex
    //~|HELP consider using `str::ends_with`

    let trivial_contains = Regex::new("foobar");
    //~^ERROR: trivial regex
    //~|HELP consider using `str::contains`

    let trivial_contains = Regex::new(NOT_A_REAL_REGEX);
    //~^ERROR: trivial regex
    //~|HELP consider using `str::contains`

    // unlikely corner cases
    let trivial_empty = Regex::new("");
    //~^ERROR: trivial regex
    //~|HELP the regex is unlikely to be useful

    let trivial_empty = Regex::new("^$");
    //~^ERROR: trivial regex
    //~|HELP consider using `str::is_empty`

    // non-trivial regexes
    let non_trivial_eq = Regex::new("^foo|bar$");
    let non_trivial_starts_with = Regex::new("^foo|bar");
    let non_trivial_ends_with = Regex::new("^foo|bar");
    let non_trivial_ends_with = Regex::new("foo|bar");
}

fn main() {
    syntax_error();
    trivial_regex();
}
