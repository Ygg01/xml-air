use std::io::BufReader;

use xml::lexer::{Lexer, XmlResult, Char, RestrictedChar,RequiredDecl,FixedDecl};
use xml::lexer::{PrologEnd, PrologStart, PI, CData, WhiteSpace};
use xml::lexer::{DoctypeStart, CharRef};
use xml::lexer::{Percent, NameToken, EntityType, Comment};
use xml::lexer::{GreaterBracket, LessBracket, ElementType};
use xml::lexer::{CloseTag,Eq,Star,QuestionMark,Plus,Pipe};
use xml::lexer::{LeftParen,RightParen,EmptyTag,QuotedString,Text};
use xml::lexer::{Ref, Quote, QNameToken, ImpliedDecl};
use xml::lexer::{LeftSqBracket, RightSqBracket, PCDataDecl};
use xml::lexer::{Comma,ParRef, DoctypeOpen, DoctypeClose, NotationType};
use xml::lexer::{AttlistType,NMToken};

#[test]
fn iteration() {
    let bytes = bytes!("<a>");
    let mut r = BufReader::new(bytes);
    let mut lexer = Lexer::from_reader(&mut r);
    for token in lexer.tokens() {
    }

    assert_eq!(None,                lexer.pull());
}

#[test]
fn whitespace() {
    let str1 = bytes!("  \t\n  a");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Ok(WhiteSpace(~"  \t\n  "))),  lexer.pull());
    assert_eq!(6,                                  lexer.col);
    assert_eq!(1,                                  lexer.line);
    assert_eq!(Some(Ok(NameToken(~"a"))),          lexer.pull());
}

#[test]
fn pi_token() {
    let str0 = bytes!("<?php var = echo()?><?php?>");
    let mut buf0 = BufReader::new(str0);

    let mut lexer = Lexer::from_reader(&mut buf0);

    assert_eq!(Some(Ok(PI(~"php", ~"var = echo()"))),   lexer.pull());
    assert_eq!(Some(Ok(PI(~"php", ~""))),               lexer.pull());

    let str1 = bytes!("<?xml encoding = 'UTF-8'?>");
    let mut buf1 =BufReader::new(str1);

    lexer = Lexer::from_reader(&mut buf1);

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"encoding"))),        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"UTF-8"))),        lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let str3 = bytes!("<?xml standalone = 'yes'?>");
    let mut buf3 = BufReader::new(str3);

    lexer = Lexer::from_reader(&mut buf3);

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"standalone"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"yes"))),          lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let str4 = bytes!("<?xml standalone = 'no'?>");
    let mut buf4 =BufReader::new(str4);

    lexer = Lexer::from_reader(&mut buf4);

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"standalone"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"no"))),           lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let str5 = bytes!("<?xml version = '1.0'?>");
    let mut buf5 =BufReader::new(str5);

    lexer = Lexer::from_reader(&mut buf5);

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"version"))),         lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"1.0"))),          lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());
}

#[test]
fn cdata() {

    let str1  = bytes!("<![CDATA[various text data like <a>]]>!");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    assert_eq!(Some(Ok(CData(~"various text data like <a>"))),  lexer.pull());
    assert_eq!(Some(Char('!')),                     lexer.read_chr());

    let str2 = bytes!("<![C!");
    let mut read2 = BufReader::new(str2);

    lexer = Lexer::from_reader(&mut read2);

    lexer.pull();
    assert_eq!(Some(Char('C')),                     lexer.read_chr());
}


#[test]
fn eof() {
    let str1 = bytes!("a");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Char('a')),     lexer.read_chr());
    assert_eq!(None,                lexer.read_chr())
}

#[test]
/// Tests if it reads a restricted character
/// and recognize a char correctly
fn restricted_char() {
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
fn read_newline() {
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
fn comment(){
    let str1  = bytes!("<!-- Nice comments --><>");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    assert_eq!(Some(Ok(Comment(~" Nice comments "))), lexer.pull());
    assert_eq!(Some(Ok(LessBracket)), lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)), lexer.pull());
}

#[test]
fn element(){
    let str1  = bytes!("<elem attr='something &ref;bla&#35;&#x2A;'></elem><br/>");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);
    assert_eq!(Some(Ok(LessBracket)),           lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"elem"))),    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),      lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"attr"))),    lexer.pull());
    assert_eq!(Some(Ok(Eq)),                    lexer.pull());
    assert_eq!(Some(Ok(Quote)),                 lexer.pull());
    assert_eq!(Some(Ok(Text(~"something "))),   lexer.pull());
    assert_eq!(Some(Ok(Ref(~"ref"))),           lexer.pull());
    assert_eq!(Some(Ok(Text(~"bla"))),          lexer.pull());
    assert_eq!(Some(Ok(CharRef('#'))),          lexer.pull());
    assert_eq!(Some(Ok(CharRef('*'))),          lexer.pull());
    assert_eq!(Some(Ok(Quote)),                 lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),        lexer.pull());
    assert_eq!(Some(Ok(CloseTag)),              lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"elem"))),    lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),        lexer.pull());
    assert_eq!(Some(Ok(LessBracket)),           lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"br"))),      lexer.pull());
    assert_eq!(Some(Ok(EmptyTag)),              lexer.pull());
}

#[test]
fn qname(){
    let str1 = bytes!("<book:elem book:iso= '11231A'");
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);
    assert_eq!(Some(Ok(LessBracket)),                   lexer.pull());
    assert_eq!(Some(Ok(QNameToken(~"book",~"elem"))),   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(QNameToken(~"book",~"iso"))),    lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),              lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
    assert_eq!(Some(Ok(Text(~"11231A"))),               lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
}

#[test]
fn quote_terminating(){
    let str1 = bytes!("<el name=\"test");
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Ok(LessBracket)),               lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"el"))),          lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"name"))),        lexer.pull());
    assert_eq!(Some(Ok(Eq)),                        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(Text(~"test"))),             lexer.pull());
}

#[test]
fn  doctype_attlist() {
    let test_str = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (stuff|stuf2) #IMPLIED>
    ]>");
    let mut read = BufReader::new(test_str);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),               lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"test"))),        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"NOTATION"))),    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                 lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"stuff"))),       lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                      lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"stuf2"))),       lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(ImpliedDecl)),               lexer.pull())
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),            lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());

    let test_str2 = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (9stuff|-stuf2) #FIXED>
    ]>");
    let mut read2 = BufReader::new(test_str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),               lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"test"))),        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"NOTATION"))),    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                 lexer.pull());
    assert_eq!(Some(Ok(NMToken(~"9stuff"))),        lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                      lexer.pull());
    assert_eq!(Some(Ok(NMToken(~"-stuf2"))),        lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(FixedDecl)),                 lexer.pull())
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),            lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());

    let test_str3 = bytes!("<!DOCTYPE PUBLIC [
    <!ATTLIST test 'text&attr;' #REQUIRED #IMPLIED>
    ]>");
    let mut read3 = BufReader::new(test_str3);
    let mut lexer = Lexer::from_reader(&mut read3);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),               lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"test"))),        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(Text(~"text"))),             lexer.pull());
    assert_eq!(Some(Ok(Ref(~"attr"))),              lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(RequiredDecl)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(ImpliedDecl)),               lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),            lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
}

#[test]
fn doctype_el() {
    let str1 = bytes!("<!DOCTYPE stuff SYSTEM 'pubid' [
    <!ELEMENT (name|(#PCDATA,%div;))?+*>
    ]>");
    let mut read = BufReader::new(str1);
    let mut lexer =             Lexer::from_reader(&mut read);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"stuff"))),       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"SYSTEM"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"pubid"))),    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(ElementType)),               lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                 lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"name"))),        lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                      lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                 lexer.pull());
    assert_eq!(Some(Ok(PCDataDecl)),                lexer.pull());
    assert_eq!(Some(Ok(Comma)),                     lexer.pull());
    assert_eq!(Some(Ok(ParRef(~"div"))),            lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                lexer.pull());
    assert_eq!(Some(Ok(QuestionMark)),              lexer.pull());
    assert_eq!(Some(Ok(Plus)),                      lexer.pull());
    assert_eq!(Some(Ok(Star)),                      lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),            lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
}

#[test]
fn doctype_ent() {
    let str2 = bytes!("<!DOCTYPE PUBLIC [
    <!ENTITY % 'text%ent;&x;&#94;&#x7E;' PUBLIC 'quote'><![]]>
    ]>");
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(EntityType)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(Percent)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(Text(~"text"))),             lexer.pull());
    assert_eq!(Some(Ok(ParRef(~"ent"))),            lexer.pull());
    assert_eq!(Some(Ok(Ref(~"x"))),                 lexer.pull());
    assert_eq!(Some(Ok(CharRef('^'))),              lexer.pull());
    assert_eq!(Some(Ok(CharRef('~'))),              lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"quote"))),    lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(DoctypeOpen)),               lexer.pull());
    assert_eq!(Some(Ok(DoctypeClose)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
}

#[test]
fn doctype_notation() {
    let str2 = bytes!("<!DOCTYPE PUBLIC [
    <!NOTATION PUBLIC \"blabla\">
    ]>");
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    assert_eq!(Some(Ok(DoctypeStart)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(NotationType)),              lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(NameToken(~"PUBLIC"))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~" "))),          lexer.pull());
    assert_eq!(Some(Ok(QuotedString(~"blabla"))),   lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(~"\n    "))),     lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),            lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
}
