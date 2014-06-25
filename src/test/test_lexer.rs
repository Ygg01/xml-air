use std::io::BufReader;

use xml::lexer::{Lexer, Char, RestrictedChar,RequiredDecl,FixedDecl};
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
    let bytes = b"<a>";
    let mut r = BufReader::new(bytes);
    let mut lexer = Lexer::from_reader(&mut r);
    for token in lexer.tokens() {
    }

    assert_eq!(None,                lexer.pull());
}

#[test]
fn whitespace() {
    let str1 = b"  \t\n  a";
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Ok(WhiteSpace("  \t\n  ".into_string()))),  lexer.pull());
    assert_eq!(6,                                               lexer.col);
    assert_eq!(1,                                               lexer.line);
    assert_eq!(Some(Ok(NameToken("a".into_string()))),          lexer.pull());
}

#[test]
fn pi_token() {
    let str0 = b"<?php var = echo()?><?php?>";
    let mut buf0 = BufReader::new(str0);

    let mut lexer = Lexer::from_reader(&mut buf0);

    assert_eq!(Some(Ok(PI("php".into_string(), "var = echo()".into_string()))),
                lexer.pull());
    assert_eq!(Some(Ok(PI("php".into_string(), "".into_string()))),
                lexer.pull());

    let str1 = b"<?xml encoding = 'UTF-8'?>";
    let mut buf1 =BufReader::new(str1);

    lexer = Lexer::from_reader(&mut buf1);

    let ws = " ".into_string();
    let encoding = "encoding".into_string();
    let utf = "UTF-8".into_string();

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(encoding))),           lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(utf))),             lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let str3 = b"<?xml standalone = 'yes'?>";
    let mut buf3 = BufReader::new(str3);

    lexer = Lexer::from_reader(&mut buf3);

    let yes = "yes".into_string();
    let standalone = "standalone".into_string();

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(standalone.clone()))), lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(yes))),             lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let mut buf4 =BufReader::new(b"<?xml standalone = 'no'?>");

    lexer = Lexer::from_reader(&mut buf4);

    let no = "no".into_string();

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(standalone.clone()))), lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(no))),              lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());

    let str5 = b"<?xml version = '1.0'?>";
    let mut buf5 =BufReader::new(str5);

    lexer = Lexer::from_reader(&mut buf5);

    let ver = "version".into_string();
    let one = "1.0".into_string();

    assert_eq!(Some(Ok(PrologStart)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(ver))),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Eq)),                            lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(one))),             lexer.pull());
    assert_eq!(Some(Ok(PrologEnd)),                     lexer.pull());
}

#[test]
fn cdata() {
    let mut read1 = BufReader::new(b"<![CDATA[various text data like <a>]]>!");

    let mut lexer = Lexer::from_reader(&mut read1);

    let cdata = "various text data like <a>".into_string();
    assert_eq!(Some(Ok(CData(cdata))),              lexer.pull());
    assert_eq!(Some(Char('!')),                     lexer.read_chr());

    let str2 = b"<![C!";
    let mut read2 = BufReader::new(str2);

    lexer = Lexer::from_reader(&mut read2);

    lexer.pull();
    assert_eq!(Some(Char('C')),                     lexer.read_chr());
}


#[test]
fn eof() {
    let str1 = b"a";
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    assert_eq!(Some(Char('a')),     lexer.read_chr());
    assert_eq!(None,                lexer.read_chr())
}

#[test]
/// Tests if it reads a restricted character
/// and recognize a char correctly
fn restricted_char() {
    let str1 = "\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F".as_bytes();
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
    let str1  = "a\r\nt".as_bytes();
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

    let str2  = "a\rt".as_bytes();
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

    let str3  = "a\r\x85t".as_bytes();
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

    let str4  = "a\x85t".as_bytes();
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

    let str5  = "a\u2028t".as_bytes();
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
    let str1  = b"<!-- Nice comments --><>";
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    let comment = " Nice comments ".into_string();

    assert_eq!(Some(Ok(Comment(comment))),  lexer.pull());
    assert_eq!(Some(Ok(LessBracket)),       lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),    lexer.pull());
}

#[test]
fn element(){
    let str1  = b"<elem attr='something &ref;bla&#35;&#x2A;'></elem><br/>";
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    let elem = "elem".into_string();
    let ws = " ".into_string();
    let attr= "attr".into_string();
    let something = "something ".into_string();
    let refs = "ref".into_string();
    let bla = "bla".into_string();
    let br = "br".into_string();

    assert_eq!(Some(Ok(LessBracket)),               lexer.pull());
    assert_eq!(Some(Ok(NameToken(elem.clone()))),   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws))),            lexer.pull());
    assert_eq!(Some(Ok(NameToken(attr))),           lexer.pull());
    assert_eq!(Some(Ok(Eq)),                        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(Text(something))),           lexer.pull());
    assert_eq!(Some(Ok(Ref(refs))),                 lexer.pull());
    assert_eq!(Some(Ok(Text(bla))),                 lexer.pull());
    assert_eq!(Some(Ok(CharRef('#'))),              lexer.pull());
    assert_eq!(Some(Ok(CharRef('*'))),              lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(CloseTag)),                  lexer.pull());
    assert_eq!(Some(Ok(NameToken(elem))),           lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),            lexer.pull());
    assert_eq!(Some(Ok(LessBracket)),               lexer.pull());
    assert_eq!(Some(Ok(NameToken(br))),             lexer.pull());
    assert_eq!(Some(Ok(EmptyTag)),                  lexer.pull());
}

#[test]
fn qname(){
    let str1 = b"<book:elem book:iso= '11231A'";
    let mut read1 = BufReader::new(str1);

    let mut lexer = Lexer::from_reader(&mut read1);

    let elem = "elem".into_string();
    let ws = " ".into_string();
    let book= "book".into_string();
    let iso = "iso".into_string();
    let text = "11231A".into_string();

    assert_eq!(Some(Ok(LessBracket)),                       lexer.pull());
    assert_eq!(Some(Ok(QNameToken(book.clone(),elem))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(QNameToken(book.clone(),iso))),      lexer.pull());
    assert_eq!(Some(Ok(Eq)),                                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(Quote)),                             lexer.pull());
    assert_eq!(Some(Ok(Text(text))),                        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                             lexer.pull());
}

#[test]
fn quote_terminating(){
    let str1 = b"<el name=\"test";
    let mut read = BufReader::new(str1);
    let mut lexer = Lexer::from_reader(&mut read);

    let el = "el".into_string();
    let ws = " ".into_string();
    let name = "name".into_string();
    let test = "test".into_string();

    assert_eq!(Some(Ok(LessBracket)),               lexer.pull());
    assert_eq!(Some(Ok(NameToken(el))),             lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws))),            lexer.pull());
    assert_eq!(Some(Ok(NameToken(name))),           lexer.pull());
    assert_eq!(Some(Ok(Eq)),                        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                     lexer.pull());
    assert_eq!(Some(Ok(Text(test))),                lexer.pull());
}

#[test]
fn  doctype_attlist() {
    let test_str = b"<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (stuff|stuf2) #IMPLIED>
    ]>";
    let mut read = BufReader::new(test_str);
    let mut lexer = Lexer::from_reader(&mut read);

    let ws = " ".into_string();
    let public = "PUBLIC".into_string();
    let newl = "\n    ".into_string();
    let test = "test".into_string();
    let notation = "NOTATION".into_string();
    let stuff = "stuff".into_string();
    let stuff2 = "stuf2".into_string();

    assert_eq!(Some(Ok(DoctypeStart)),                      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),         lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),          lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),                       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(NameToken(test.clone()))),           lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(NameToken(notation.clone()))),       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                         lexer.pull());
    assert_eq!(Some(Ok(NameToken(stuff.clone()))),          lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                              lexer.pull());
    assert_eq!(Some(Ok(NameToken(stuff2.clone()))),         lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(ImpliedDecl)),                       lexer.pull())
    assert_eq!(Some(Ok(GreaterBracket)),                    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),          lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),                    lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                    lexer.pull());

    let test_str2 = b"<!DOCTYPE PUBLIC [
    <!ATTLIST test NOTATION (9stuff|-stuf2) #FIXED>
    ]>";
    let mut read2 = BufReader::new(test_str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    let stuff9 = "9stuff".into_string();
    let stuf2 = "-stuf2".into_string();

    assert_eq!(Some(Ok(DoctypeStart)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                 lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(test.clone()))),       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(notation.clone()))),   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                     lexer.pull());
    assert_eq!(Some(Ok(NMToken(stuff9.clone()))),       lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                          lexer.pull());
    assert_eq!(Some(Ok(NMToken(stuf2.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(FixedDecl)),                     lexer.pull())
    assert_eq!(Some(Ok(GreaterBracket.clone())),        lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),                lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());

    let test_str3 = b"<!DOCTYPE PUBLIC [
    <!ATTLIST test 'text&attr;' #REQUIRED #IMPLIED>
    ]>";
    let mut read3 = BufReader::new(test_str3);
    let mut lexer = Lexer::from_reader(&mut read3);

    let text = "text".into_string();
    let attr = "attr".into_string();

    assert_eq!(Some(Ok(DoctypeStart)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                 lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(AttlistType)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(test.clone()))),       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
    assert_eq!(Some(Ok(Text(text.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(Ref(attr.clone()))),             lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(RequiredDecl)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(ImpliedDecl)),                   lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),                lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
}

#[test]
fn doctype_el() {
    let str1 = b"<!DOCTYPE stuff SYSTEM 'pubid' [
    <!ELEMENT (name|(#PCDATA,%div;))?+*>
    ]>";
    let mut read = BufReader::new(str1);
    let mut lexer =             Lexer::from_reader(&mut read);

    let ws = " ".into_string();
    let newl = "\n    ".into_string();
    let stuff = "stuff".into_string();
    let system = "SYSTEM".into_string();
    let pubid = "pubid".into_string();
    let name = "name".into_string();
    let div = "div".into_string();

    assert_eq!(Some(Ok(DoctypeStart)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(stuff.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(system.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(pubid.clone()))),   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                 lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(ElementType)),                   lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                     lexer.pull());
    assert_eq!(Some(Ok(NameToken(name.clone()))),       lexer.pull());
    assert_eq!(Some(Ok(Pipe)),                          lexer.pull());
    assert_eq!(Some(Ok(LeftParen)),                     lexer.pull());
    assert_eq!(Some(Ok(PCDataDecl)),                    lexer.pull());
    assert_eq!(Some(Ok(Comma)),                         lexer.pull());
    assert_eq!(Some(Ok(ParRef(div.clone()))),           lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                    lexer.pull());
    assert_eq!(Some(Ok(RightParen)),                    lexer.pull());
    assert_eq!(Some(Ok(QuestionMark)),                  lexer.pull());
    assert_eq!(Some(Ok(Plus)),                          lexer.pull());
    assert_eq!(Some(Ok(Star)),                          lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),                lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
}

#[test]
fn doctype_ent() {
    let str2 = b"<!DOCTYPE PUBLIC [
    <!ENTITY % 'text%ent;&x;&#94;&#x7E;' PUBLIC 'quote'><![]]>
    ]>";
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    let ws = " ".into_string();
    let newl = "\n    ".into_string();
    let public = "PUBLIC".into_string();
    let text = "text".into_string();
    let ent = "ent".into_string();
    let x = "x".into_string();
    let quote = "quote".into_string();


    assert_eq!(Some(Ok(DoctypeStart)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                 lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(EntityType)),                    lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Percent)),                       lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
    assert_eq!(Some(Ok(Text(text.clone()))),            lexer.pull());
    assert_eq!(Some(Ok(ParRef(ent.clone()))),           lexer.pull());
    assert_eq!(Some(Ok(Ref(x))),                        lexer.pull());
    assert_eq!(Some(Ok(CharRef('^'))),                  lexer.pull());
    assert_eq!(Some(Ok(CharRef('~'))),                  lexer.pull());
    assert_eq!(Some(Ok(Quote)),                         lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(quote.clone()))),   lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
    assert_eq!(Some(Ok(DoctypeOpen)),                   lexer.pull());
    assert_eq!(Some(Ok(DoctypeClose)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
}

#[test]
fn doctype_notation() {
    let str2 = b"<!DOCTYPE PUBLIC [
    <!NOTATION PUBLIC \"blabla\">
    ]>";
    let mut read2 = BufReader::new(str2);
    let mut lexer = Lexer::from_reader(&mut read2);

    let ws = " ".into_string();
    let newl = "\n    ".into_string();
    let public = "PUBLIC".into_string();
    let blabla = "blabla".into_string();

    assert_eq!(Some(Ok(DoctypeStart)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(LeftSqBracket)),                 lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(NotationType)),                  lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(NameToken(public.clone()))),     lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(ws.clone()))),        lexer.pull());
    assert_eq!(Some(Ok(QuotedString(blabla.clone()))),  lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
    assert_eq!(Some(Ok(WhiteSpace(newl.clone()))),      lexer.pull());
    assert_eq!(Some(Ok(RightSqBracket)),                lexer.pull());
    assert_eq!(Some(Ok(GreaterBracket)),                lexer.pull());
}
