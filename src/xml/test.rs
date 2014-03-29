#[cfg(test)]

use util::is_restricted;
use lexer::{Lexer, Char, RestrictedChar,RequiredDecl,FixedDecl};
use lexer::{PrologEnd, PrologStart, PI, CData, WhiteSpace};
use lexer::{DoctypeStart, CharRef};
use lexer::{Percent, NameToken, EntityType, Comment};
use lexer::{GreaterBracket, LessBracket, ElementType};
use lexer::{CloseTag,Eq,Star,QuestionMark,Plus,Pipe};
use lexer::{LeftParen,RightParen,EmptyTag,QuotedString,Text};
use lexer::{Ref, Quote, QNameToken, ImpliedDecl};
use lexer::{LeftSqBracket, RightSqBracket, PCDataDecl};
use lexer::{Comma,ParRef, DoctypeOpen, DoctypeClose, NotationType};
use lexer::{AttlistType,NMToken};
use std::io::BufReader;

mod lexer;
mod util;

#[test]
fn util_is_restricted(){
    assert_eq!(true, is_restricted(&'\x0B'));
    assert_eq!(true, is_restricted(&'\x02'));
    assert_eq!(true, is_restricted(&'\x0C'));
    assert_eq!(true, is_restricted(&'\x0F'));
    assert_eq!(true, is_restricted(&'\x1F'));
    assert_eq!(true, is_restricted(&'\x7F'));
    assert_eq!(true, is_restricted(&'\x84'));
    assert_eq!(true, is_restricted(&'\x86'));
    assert_eq!(true, is_restricted(&'\x9A'));
    assert_eq!(true, is_restricted(&'\U0001FFFE'));
    assert_eq!(true, is_restricted(&'\U0001FFFF'));
    assert_eq!(true, is_restricted(&'\U0002FFFE'));
    assert_eq!(true, is_restricted(&'\U0002FFFF'));
    assert_eq!(true, is_restricted(&'\U0003FFFE'));
    assert_eq!(true, is_restricted(&'\U0003FFFF'));
    assert_eq!(true, is_restricted(&'\U0004FFFE'));
    assert_eq!(true, is_restricted(&'\U0004FFFF'));
    assert_eq!(true, is_restricted(&'\U0005FFFE'));
    assert_eq!(true, is_restricted(&'\U0005FFFF'));
    assert_eq!(true, is_restricted(&'\U0006FFFE'));
    assert_eq!(true, is_restricted(&'\U0006FFFF'));
    assert_eq!(true, is_restricted(&'\U0007FFFE'));
    assert_eq!(true, is_restricted(&'\U0007FFFF'));
    assert_eq!(true, is_restricted(&'\U0008FFFE'));
    assert_eq!(true, is_restricted(&'\U0008FFFF'));
    assert_eq!(true, is_restricted(&'\U0009FFFE'));
    assert_eq!(true, is_restricted(&'\U0009FFFF'));
    assert_eq!(true, is_restricted(&'\U000AFFFE'));
    assert_eq!(true, is_restricted(&'\U000AFFFF'));
    assert_eq!(true, is_restricted(&'\U000BFFFE'));
    assert_eq!(true, is_restricted(&'\U000BFFFF'));
    assert_eq!(true, is_restricted(&'\U000CFFFE'));
    assert_eq!(true, is_restricted(&'\U000CFFFF'));
    assert_eq!(true, is_restricted(&'\U000DFFFE'));
    assert_eq!(true, is_restricted(&'\U000DFFFF'));
    assert_eq!(true, is_restricted(&'\U000EFFFE'));
    assert_eq!(true, is_restricted(&'\U000EFFFF'));
    assert_eq!(true, is_restricted(&'\U000FFFFE'));
    assert_eq!(true, is_restricted(&'\U000FFFFF'));
}

#[test]
fn lexer_iteration() {
    let bytes = bytes!("<a>");
    let mut r = BufReader::new(bytes);
    let mut lexer = Lexer::from_reader(&mut r);
    for token in lexer.tokens() {
    }

    assert_eq!(None,                lexer.pull());
}

#[test]
fn lexer_whitespace() {
    let str1 = bytes!("  \t\n  a");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(WhiteSpace(~"  \t\n  ")),      lexer.pull());
    assert_eq!(6,                                  lexer.col);
    assert_eq!(1,                                  lexer.line);
    assert_eq!(Some(NameToken(~"a")),              lexer.pull());
}

#[test]
fn lexer_pi_token() {
    let str0 = bytes!("<?php var = echo()?><?php?>");
    let mut buf0 = BufReader::new(str0);

    let mut lexer = Lexer::from_reader(&mut buf0);

    assert_eq!(Some(PI(~"php", ~"var = echo()")),   lexer.pull());
    assert_eq!(Some(PI(~"php", ~"")),               lexer.pull());

    let str1 = bytes!("<?xml encoding = 'UTF-8'?>");
    let mut buf1 =BufReader::new(str1);

    lexer = Lexer::from_reader(&mut buf1);

    assert_eq!(Some(PrologStart),                   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(NameToken(~"encoding")),        lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(Eq),                            lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(QuotedString(~"UTF-8")),            lexer.pull());
    assert_eq!(Some(PrologEnd),                     lexer.pull());

    let str3 = bytes!("<?xml standalone = 'yes'?>");
    let mut buf3 = BufReader::new(str3);

    lexer = Lexer::from_reader(&mut buf3);

    assert_eq!(Some(PrologStart),                   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(NameToken(~"standalone")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(Eq),                            lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(QuotedString(~"yes")),          lexer.pull());
    assert_eq!(Some(PrologEnd),                     lexer.pull());

    let str4 = bytes!("<?xml standalone = 'no'?>");
    let mut buf4 =BufReader::new(str4);

    lexer = Lexer::from_reader(&mut buf4);

    assert_eq!(Some(PrologStart),                   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(NameToken(~"standalone")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(Eq),                            lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(QuotedString(~"no")),           lexer.pull());
    assert_eq!(Some(PrologEnd),                     lexer.pull());

    let str5 = bytes!("<?xml version = '1.0'?>");
    let mut buf5 =BufReader::new(str5);

    lexer = Lexer::from_reader(&mut buf5);

    assert_eq!(Some(PrologStart),                   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(NameToken(~"version")),         lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(Eq),                            lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(QuotedString(~"1.0")),          lexer.pull());
    assert_eq!(Some(PrologEnd),                     lexer.pull());
}

#[test]
fn lexer_cdata() {

    let str1  = bytes!("<![CDATA[various text data like <a>]]>!");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    assert_eq!(Some(CData(~"various text data like <a>")),  lexer.pull());
    assert_eq!(Some(Char('!')),                     lexer.read_chr());

    let str2 = bytes!("<![C!");
    let mut read2 = BufReader::new(str2);

    lexer = Lexer::from_reader(&mut read2);

    lexer.pull();
    assert_eq!(Some(Char('C')),                     lexer.read_chr());
}


#[test]
fn lexer_eof() {
    let str1 = bytes!("a");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Char('a')),     lexer.read_chr());
    assert_eq!(None,                lexer.read_chr())
}

#[test]
/// Tests if it reads a restricted character
/// and recognize a char correctly
fn lexer_restricted_char() {
    let str1 = bytes!("\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(RestrictedChar('\x01')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x04')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x08')),      lexer.read_chr());
    assert_eq!(Some(Char('a')),                   lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x0B')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x0C')),      lexer.read_chr());
    assert_eq!(Some(Char('b')),                   lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x0E')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x10')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x1F')),      lexer.read_chr());
    assert_eq!(Some(Char('c')),                   lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x7F')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x80')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x84')),      lexer.read_chr());
    assert_eq!(Some(Char('d')),                   lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x86')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x90')),      lexer.read_chr());
    assert_eq!(Some(RestrictedChar('\x9F')),      lexer.read_chr());
}

#[test]
fn lexer_read_newline() {
    let str1  = bytes!("a\r\nt");
    let mut read1 = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read1);

    assert_eq!(Some(Char('a')), lexer.read_chr());
    assert_eq!(1,               lexer.line);
    assert_eq!(1,               lexer.col);
    assert_eq!(Some(Char('\n')),lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(0,               lexer.col);
    assert_eq!(Some(Char('t')), lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(1,               lexer.col);

    let str2  = bytes!("a\rt");
    let mut read2 = BufReader::new(str2);
    lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(Char('a')), lexer.read_chr());
    assert_eq!(1,               lexer.line);
    assert_eq!(1,               lexer.col);
    assert_eq!(Some(Char('\n')),lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(0,               lexer.col);
    assert_eq!(Some(Char('t')), lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(1,               lexer.col);

    let str3  = bytes!("a\r\x85t");
    let mut read3 = BufReader::new(str3);
    lexer = Lexer::from_reader(&mut read3);

    assert_eq!(Some(Char('a')),     lexer.read_chr());
    assert_eq!(1,                   lexer.line);
    assert_eq!(1,                   lexer.col);
    assert_eq!(Some(Char('\n')),    lexer.read_chr());
    assert_eq!(2,                   lexer.line);
    assert_eq!(0,                   lexer.col);
    assert_eq!(Some(Char('t')),     lexer.read_chr());
    assert_eq!(2,                   lexer.line);
    assert_eq!(1,                   lexer.col);

    let str4  = bytes!("a\x85t");
    let mut read4 = BufReader::new(str4);
    let mut lexer = Lexer::from_reader(&mut read4);

    assert_eq!(Some(Char('a')),     lexer.read_chr());
    assert_eq!(1,                   lexer.line);
    assert_eq!(1,                   lexer.col);
    assert_eq!(Some(Char('\n')),    lexer.read_chr());
    assert_eq!(2,                   lexer.line);
    assert_eq!(0,                   lexer.col);
    assert_eq!(Some(Char('t')),     lexer.read_chr());
    assert_eq!(2,                   lexer.line);
    assert_eq!(1,                   lexer.col);

    let str5  = bytes!("a\u2028t");
    let mut read5 = BufReader::new(str5);
    let mut lexer = Lexer::from_reader(&mut read5);

    assert_eq!(Some(Char('a')), lexer.read_chr());
    assert_eq!(1,               lexer.line);
    assert_eq!(1,               lexer.col);
    assert_eq!(Some(Char('\n')),lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(0,               lexer.col);
    assert_eq!(Some(Char('t')), lexer.read_chr());
    assert_eq!(2,               lexer.line);
    assert_eq!(1,               lexer.col);
}

#[test]
fn lexer_comment(){
    let str1  = bytes!("<!-- Nice comments --><>");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    assert_eq!(Some(Comment(~" Nice comments ")), lexer.pull());
    assert_eq!(Some(LessBracket), lexer.pull());
    assert_eq!(Some(GreaterBracket), lexer.pull());
}

#[test]
fn lexer_element(){
    let str1  = bytes!("<elem attr='something &ref;bla&#35;&#x2A;'></elem><br/>");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);
    assert_eq!(Some(LessBracket),           lexer.pull());
    assert_eq!(Some(NameToken(~"elem")),    lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),      lexer.pull());
    assert_eq!(Some(NameToken(~"attr")),    lexer.pull());
    assert_eq!(Some(Eq),                    lexer.pull());
    assert_eq!(Some(Quote),                 lexer.pull());
    assert_eq!(Some(Text(~"something ")),   lexer.pull());
    assert_eq!(Some(Ref(~"ref")),           lexer.pull());
    assert_eq!(Some(Text(~"bla")),          lexer.pull());
    assert_eq!(Some(CharRef('#')),          lexer.pull());
    assert_eq!(Some(CharRef('*')),          lexer.pull());
    assert_eq!(Some(Quote),                 lexer.pull());
    assert_eq!(Some(GreaterBracket),        lexer.pull());
    assert_eq!(Some(CloseTag),              lexer.pull());
    assert_eq!(Some(NameToken(~"elem")),    lexer.pull());
    assert_eq!(Some(GreaterBracket),        lexer.pull());
    assert_eq!(Some(LessBracket),           lexer.pull());
    assert_eq!(Some(NameToken(~"br")),      lexer.pull());
    assert_eq!(Some(EmptyTag),              lexer.pull());
}

#[test]
fn lexer_qname(){
    let str1 = bytes!("<book:elem book:iso= '11231A'");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);
    assert_eq!(Some(LessBracket),                   lexer.pull());
    assert_eq!(Some(QNameToken(~"book",~"elem")),   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(QNameToken(~"book",~"iso")),    lexer.pull());
    assert_eq!(Some(Eq),                            lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
    assert_eq!(Some(Quote),                         lexer.pull());
    assert_eq!(Some(Text(~"11231A")),               lexer.pull());
    assert_eq!(Some(Quote),                         lexer.pull());
}

#[test]
fn lexer_quote_terminating(){
    let str1 = bytes!("<el name=\"test");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(LessBracket),               lexer.pull());
    assert_eq!(Some(NameToken(~"el")),          lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"name")),        lexer.pull());
    assert_eq!(Some(Eq),                        lexer.pull());
    assert_eq!(Some(Quote),                     lexer.pull());
    assert_eq!(Some(Text(~"test")),             lexer.pull());
}

#[test]
fn  lexer_doctype_attlist() {
    let test_str = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (stuff|stuf2) #IMPLIED>
    ]>");
    let mut read = BufReader::new(test_str);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(AttlistType),               lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"test")),        lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"NOTATION")),    lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftParen),                 lexer.pull());
    assert_eq!(Some(NameToken(~"stuff")),       lexer.pull());
    assert_eq!(Some(Pipe),                      lexer.pull());
    assert_eq!(Some(NameToken(~"stuf2")),       lexer.pull());
    assert_eq!(Some(RightParen),                lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(ImpliedDecl),               lexer.pull())
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(RightSqBracket),            lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());

    let test_str2 = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (9stuff|-stuf2) #FIXED>
    ]>");
    let mut read2 = BufReader::new(test_str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(AttlistType),               lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"test")),        lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"NOTATION")),    lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftParen),                 lexer.pull());
    assert_eq!(Some(NMToken(~"9stuff")),        lexer.pull());
    assert_eq!(Some(Pipe),                      lexer.pull());
    assert_eq!(Some(NMToken(~"-stuf2")),        lexer.pull());
    assert_eq!(Some(RightParen),                lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(FixedDecl),                 lexer.pull())
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(RightSqBracket),            lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());

    let test_str3 = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test 'text&attr;' #REQUIRED #IMPLIED>
    ]>");
    let mut read3 = BufReader::new(test_str3);
    let mut lexer = Lexer::from_reader(&mut read3);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(AttlistType),               lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"test")),        lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(Quote),                     lexer.pull());
    assert_eq!(Some(Text(~"text")),             lexer.pull());
    assert_eq!(Some(Ref(~"attr")),              lexer.pull());
    assert_eq!(Some(Quote),                     lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(RequiredDecl),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(ImpliedDecl),               lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(RightSqBracket),            lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
}

#[test]
fn lexer_doctype_el() {
    let str1 = bytes!("<!DOCTYPE stuff SYSTEM 'pubid' [
    <!ELEMENT (name|(#PCDATA,%div;))?+*>
    ]>");
    let mut read = BufReader::new(str1);
    let mut lexer =             Lexer::from_reader(&mut read);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"stuff")),       lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"SYSTEM")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(QuotedString(~"pubid")),    lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(ElementType),               lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftParen),                 lexer.pull());
    assert_eq!(Some(NameToken(~"name")),        lexer.pull());
    assert_eq!(Some(Pipe),                      lexer.pull());
    assert_eq!(Some(LeftParen),                 lexer.pull());
    assert_eq!(Some(PCDataDecl),                lexer.pull());
    assert_eq!(Some(Comma),                     lexer.pull());
    assert_eq!(Some(ParRef(~"div")),            lexer.pull());
    assert_eq!(Some(RightParen),                lexer.pull());
    assert_eq!(Some(RightParen),                lexer.pull());
    assert_eq!(Some(QuestionMark),              lexer.pull());
    assert_eq!(Some(Plus),                      lexer.pull());
    assert_eq!(Some(Star),                      lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(RightSqBracket),            lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
}

#[test]
fn lexer_doctype_ent() {
    let str2 = bytes!("<!DOCTYPE PUBLIC [
    <!ENTITY % 'text%ent;&x;&#94;&#x7E;' PUBLIC 'quote'><![]]>
    ]>");
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(EntityType),                lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(Percent),                   lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(Quote),                     lexer.pull());
    assert_eq!(Some(Text(~"text")),             lexer.pull());
    assert_eq!(Some(ParRef(~"ent")),            lexer.pull());
    assert_eq!(Some(Ref(~"x")),                 lexer.pull());
    assert_eq!(Some(CharRef('^')),              lexer.pull());
    assert_eq!(Some(CharRef('~')),              lexer.pull());
    assert_eq!(Some(Quote),                     lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(QuotedString(~"quote")),    lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(DoctypeOpen),               lexer.pull());
    assert_eq!(Some(DoctypeClose),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
}

#[test]
fn lexer_doctype_notation() {
    let str2 = bytes!("<!DOCTYPE PUBLIC [
    <!NOTATION PUBLIC \"blabla\">
    ]>");
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(DoctypeStart),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(LeftSqBracket),             lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(NotationType),              lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
    assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
    assert_eq!(Some(QuotedString(~"blabla")),   lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
    assert_eq!(Some(WhiteSpace(~"\n    ")), lexer.pull());
    assert_eq!(Some(RightSqBracket),            lexer.pull());
    assert_eq!(Some(GreaterBracket),            lexer.pull());
}
