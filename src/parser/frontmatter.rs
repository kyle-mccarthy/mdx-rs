use super::Parse;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till1, take_until1},
    character::complete::not_line_ending,
    combinator::{eof, verify},
    multi::many_till,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Key<'a>(pub &'a str);

impl<'a> Parse<'a> for Key<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, key) = verify(take_until1(":"), |s: &str| !s.contains('\n'))(input)?;
        let rest = rest.trim_start();

        let (rest, _) = take(2usize)(rest)?;

        Ok((rest, Self(key)))
    }
}

#[derive(Debug, PartialEq)]
pub struct Indent;

impl<'a> Parse<'a> for Indent {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag("  ")(input)?;
        Ok((rest, Self))
    }
}

#[derive(Debug, PartialEq)]
pub struct ListItem;

impl<'a> Parse<'a> for ListItem {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, _) = tag("- ")(input)?;

        Ok((rest, Self))
    }
}

#[derive(Debug, PartialEq)]
pub struct LineBreak;

impl<'a> Parse<'a> for LineBreak {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag("\n")(input)?;
        Ok((rest, Self))
    }
}

#[derive(Debug, PartialEq)]
pub struct Text<'a>(pub &'a str);

impl<'a> Parse<'a> for Text<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, contents) = not_line_ending(input)?;
        Ok((rest, Self(contents)))
    }
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Key(Key<'a>),
    ListItem(ListItem),
    Indent(Indent),
    LineBreak(LineBreak),
    Text(Text<'a>),
}

impl<'a> Parse<'a> for Token<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        alt((
            LineBreak::parse_token,
            Indent::parse_token,
            ListItem::parse_token,
            Key::parse_token,
            Text::parse_token,
        ))(input)
    }
}

#[derive(Debug, PartialEq)]
pub struct Tokens<'a>(pub Vec<Token<'a>>);

impl<'a> Parse<'a> for Tokens<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, (tokens, _)) = many_till(Token::parse, eof)(input)?;
        Ok((rest, Tokens(tokens)))
    }
}

impl<'a> From<Key<'a>> for Token<'a> {
    fn from(key: Key<'a>) -> Self {
        Token::Key(key)
    }
}

impl<'a> From<ListItem> for Token<'a> {
    fn from(list_item: ListItem) -> Self {
        Token::ListItem(list_item)
    }
}

impl<'a> From<Indent> for Token<'a> {
    fn from(indent: Indent) -> Self {
        Token::Indent(indent)
    }
}

impl<'a> From<LineBreak> for Token<'a> {
    fn from(line_break: LineBreak) -> Self {
        Token::LineBreak(line_break)
    }
}

impl<'a> From<Text<'a>> for Token<'a> {
    fn from(text: Text<'a>) -> Self {
        Token::Text(text)
    }
}

pub trait ParseToken<'a> {
    fn parse_token(input: &'a str) -> IResult<&'a str, Token<'a>>;
}

impl<'a, T> ParseToken<'a> for T
where
    T: Parse<'a> + Into<Token<'a>>,
{
    fn parse_token(input: &'a str) -> IResult<&'a str, Token<'a>> {
        let (rest, this) = T::parse(input)?;
        let token: Token = this.into();
        Ok((rest, token))
    }
}

#[derive(Debug)]
pub struct Map<'a>(pub Vec<(Key<'a>, Value<'a>)>);

#[derive(Debug)]
pub struct List<'a>(pub Vec<Value<'a>>);

#[derive(Debug)]
pub enum Value<'a> {
    Text(Text<'a>),
    List(List<'a>),
    Map(Map<'a>),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub struct Document<'a>(pub Vec<Value<'a>>);

impl<'a> Document<'a> {
    pub fn from_tokens(&self) -> Result<Self, Error> {
        todo!()
    }
}

#[cfg(test)]
mod test_frontmatter {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_key() {
        let input = "key: value";

        let (rest, key) = Key::parse(input).unwrap();

        assert_eq!(rest, "value");
        assert_eq!(key, Key("key"));
    }

    #[test]
    fn test_parse_list_item() {
        let input = "- list item\n";

        let (rest, _) = ListItem::parse(input).unwrap();

        assert_eq!(rest, "list item\n");
    }

    #[test]
    fn test_parse_tokens() {
        let input = indoc! {"
            title: the title
            keywords: 
              - item 1
              - item 2
        "};

        let tokens = Tokens::parse(input).unwrap();
        dbg!(tokens);

        let input = indoc! {"
            author:
              - Author one
              - Author two
            author:
              - name: Author one
                affiliation: University X
              - name: Author two
                affiliation: University Y
        "};

        let tokens = Tokens::parse(input).unwrap();
        dbg!(tokens);
    }
}
