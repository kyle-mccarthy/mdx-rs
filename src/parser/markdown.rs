use std::ops::Index;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until, take_until1, take_while1},
    character::complete::{digit1, line_ending, not_line_ending},
    combinator::{all_consuming, eof},
    multi::{many0, many1, many_till},
    IResult,
};

use super::Parse;

/// Parses a line of test, discarding the new line sequence and returning the line and remaining
/// text.
pub fn parse_line(input: &str) -> IResult<&str, &str> {
    let (rest, line) = not_line_ending(input)?;
    let (rest, _) = line_ending(rest)?;
    Ok((rest, line))
}


#[derive(Debug, PartialEq)]
pub struct Heading<'a> {
    pub level: u8,
    pub text: &'a str,
}

impl<'a> Parse<'a> for Heading<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_level(i: &str) -> IResult<&str, u8> {
            let (rest, level): (&str, &str) = take_while1(|c: char| c == '#')(i)?;
            let level: usize = level.len();
            let level = level as u8;
            Ok((rest, level))
        }

        // get the "level" of the heading
        let (rest, level) = parse_level(input)?;
        let (rest, text) = parse_line(rest)?;

        // return the heading with the remaining text
        Ok((
            rest,
            Heading {
                level,
                text: text.trim(),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct CodeBlock<'a> {
    pub lang: Option<&'a str>,
    pub contents: &'a str,
}

impl<'a> Parse<'a> for CodeBlock<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_start(input: &str) -> IResult<&str, Option<&str>> {
            let (rest, _) = tag("```")(input)?;

            let (rest, lang) = parse_line(rest)?;
            let lang = lang.trim();

            let lang = match lang.len() {
                0 => None,
                _ => Some(lang),
            };

            Ok((rest, lang))
        }

        fn parse_content(input: &str) -> IResult<&str, &str> {
            let (rest, content) = take_until("```")(input)?;
            // get rid of the new line
            let (_, content) = not_line_ending(content)?;
            // get rid of the closing ```
            let (rest, _) = tag("```")(rest)?;
            Ok((rest, content))
        }

        let (rest, lang) = parse_start(input)?;
        let (rest, contents) = parse_content(rest)?;

        Ok((rest, CodeBlock { lang, contents }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Link<'a> {
    pub text: &'a str,
    pub url: &'a str,
}

impl<'a> Parse<'a> for Link<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        // parse the text
        let (rest, _) = tag("[")(input)?;
        let (rest, text) = take_until("]")(rest)?;
        let (rest, _) = tag("]")(rest)?;
        // parse the url
        let (rest, _) = tag("(")(rest)?;
        let (rest, url) = take_until(")")(rest)?;
        let (rest, _) = tag(")")(rest)?;
        Ok((rest, Self { text, url }))
    }
}

impl<'a> Link<'a> {
    pub fn parse_into_text_block(input: &'a str) -> IResult<&str, TextBlockItem> {
        let (rest, inner) = Self::parse(input)?;
        Ok((rest, TextBlockItem::Link(inner)))
    }
}

#[derive(Debug, PartialEq)]
pub struct Image<'a> {
    pub alt: &'a str,
    pub source: &'a str,
}

impl<'a> Parse<'a> for Image<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        // parse the alt
        let (rest, _) = tag("![")(input)?;
        let (rest, alt) = take_until("]")(rest)?;
        let (rest, _) = tag("]")(rest)?;
        // parse the source
        let (rest, _) = tag("(")(rest)?;
        let (rest, source) = take_until(")")(rest)?;
        let (rest, _) = tag(")")(rest)?;
        Ok((rest, Self { alt, source }))
    }
}

#[derive(Debug, PartialEq)]
pub struct UnorderedList<'a> {
    pub items: Vec<&'a str>,
}

impl<'a> Parse<'a> for UnorderedList<'a> {
    /// Parse the input into an unordered list.
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_list_item(i: &str) -> IResult<&str, &str> {
            let (rest, _) = tag("- ")(i)?;
            let (rest, item) = parse_line(rest)?;
            Ok((rest, item.trim()))
        }

        let (rest, items) = many1(parse_list_item)(input)?;

        Ok((rest, UnorderedList { items }))
    }
}

impl<'a> UnorderedList<'a> {
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a> Index<usize> for UnorderedList<'a> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.items[index]
    }
}

impl<'a> IntoIterator for UnorderedList<'a> {
    type Item = &'a str;
    type IntoIter = <Vec<&'a str> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub struct OrderedList<'a> {
    pub items: Vec<&'a str>,
}

impl<'a> Parse<'a> for OrderedList<'a> {
    /// Parse the input into an ordered list.
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_list_item(i: &str) -> IResult<&str, &str> {
            let (rest, _) = digit1(i)?;
            let (rest, _) = tag(". ")(rest)?;
            let (rest, item) = parse_line(rest)?;
            Ok((rest, item.trim()))
        }

        let (rest, items) = many1(parse_list_item)(input)?;

        Ok((rest, OrderedList { items }))
    }
}

impl<'a> OrderedList<'a> {
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a> IntoIterator for OrderedList<'a> {
    type Item = &'a str;
    type IntoIter = <Vec<&'a str> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> Index<usize> for OrderedList<'a> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.items[index]
    }
}

#[derive(Debug, PartialEq)]
pub struct Task<'a> {
    pub text: &'a str,
    pub completed: bool,
}

impl<'a> Parse<'a> for Task<'a> {
    /// Parse the input as a task. Expects the input to include the leading -.
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_completed(input: &str) -> IResult<&str, bool> {
            let (rest, _) = tag("- [x] ")(input)?;
            Ok((rest, true))
        }

        fn parse_incomplete(input: &str) -> IResult<&str, bool> {
            let (rest, _) = tag("- [ ] ")(input)?;
            Ok((rest, false))
        }

        let (rest, completed) = alt((parse_completed, parse_incomplete))(input)?;
        let (rest, text) = parse_line(rest)?;

        Ok((rest, Self { text, completed }))
    }
}

#[derive(Debug, PartialEq)]
pub struct TaskList<'a> {
    pub tasks: Vec<Task<'a>>,
}

impl<'a> Parse<'a> for TaskList<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, tasks) = many1(Task::parse)(input)?;
        Ok((rest, Self { tasks }))
    }
}
impl<'a> TaskList<'a> {
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl<'a> Index<usize> for TaskList<'a> {
    type Output = Task<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tasks[index]
    }
}

impl<'a> IntoIterator for TaskList<'a> {
    type Item = Task<'a>;
    type IntoIter = <Vec<Task<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tasks.into_iter()
    }
}

/// A reference to a footnote.
///
/// # Example
/// ```markdown
/// [^note]
/// ```
#[derive(Debug, PartialEq)]
pub struct FootnoteRef<'a> {
    pub name: &'a str,
}

impl<'a> FootnoteRef<'a> {
    pub fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, _) = tag("[^")(input)?;
        let (rest, name) = take_until("]")(rest)?;
        let (rest, _) = tag("]")(rest)?;
        Ok((rest, Self { name }))
    }

    pub fn parse_into_text_block(input: &'a str) -> IResult<&str, TextBlockItem> {
        let (rest, inner) = Self::parse(input)?;
        Ok((rest, TextBlockItem::FootnoteRef(inner)))
    }
}

/// A footnote.
///
/// # Example
/// ```markdown
/// [^1]: My reference.
/// ```
#[derive(Debug, PartialEq)]
pub struct Footnote<'a> {
    pub name: &'a str,
    pub text: Vec<&'a str>,
}

impl<'a> Parse<'a> for Footnote<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        fn parse_extra_lines(input: &str) -> IResult<&str, &str> {
            let (rest, _) = tag("  ")(input)?;
            let (rest, line) = parse_line(rest)?;
            Ok((rest, line))
        }

        let (rest, _) = tag("[^")(input)?;
        let (rest, name) = take_until("]:")(rest)?;
        let (rest, _) = tag("]:")(rest)?;
        let (rest, text) = parse_line(rest)?;

        let (rest, mut lines) = many0(parse_extra_lines)(rest)?;

        lines.insert(0, text.trim());

        Ok((rest, Self { name, text: lines }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Text<'a>(pub &'a str);

impl<'a> Text<'a> {
    pub fn parse_into_text_block(input: &'a str) -> IResult<&str, TextBlockItem> {
        let (rest, text) = Self::parse(input)?;
        Ok((rest, TextBlockItem::Text(text)))
    }
}

impl<'a> Parse<'a> for Text<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, text) = take_till1(|c| c == '`' || c == '[')(input)?;

        Ok((rest, Self(text)))
    }
}

#[derive(Debug, PartialEq)]
pub enum TextBlockItem<'a> {
    Text(Text<'a>),
    FootnoteRef(FootnoteRef<'a>),
    Link(Link<'a>),
}

#[derive(Debug, PartialEq)]
pub struct TextBlock<'a> {
    pub contents: Vec<TextBlockItem<'a>>,
}

impl<'a> TextBlock<'a> {
    pub fn len(&self) -> usize {
        self.contents.len()
    }
}

impl<'a> Parse<'a> for TextBlock<'a> {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, contents) = take_until1("\n\n")(input)?;
        let (rest, _) = many1(tag("\n"))(rest)?;

        let (_, contents) = all_consuming(many1(alt((
            Text::parse_into_text_block,
            FootnoteRef::parse_into_text_block,
            Link::parse_into_text_block,
        ))))(contents)?;

        Ok((rest, Self { contents }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Newline;

impl<'a> Parse<'a> for Newline {
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (rest, _) = line_ending(input)?;
        Ok((rest, Self))
    }
}

#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    Heading(Heading<'a>),
    CodeBlock(CodeBlock<'a>),
    Link(Link<'a>),
    Image(Image<'a>),
    OrderedList(OrderedList<'a>),
    UnorderedList(UnorderedList<'a>),
    TaskList(TaskList<'a>),
    Footnote(Footnote<'a>),
    TextBlock(TextBlock<'a>),
    Newline(Newline),
}

impl<'a> From<Heading<'a>> for Block<'a> {
    fn from(heading: Heading<'a>) -> Self {
        Block::Heading(heading)
    }
}

impl<'a> From<CodeBlock<'a>> for Block<'a> {
    fn from(code_block: CodeBlock<'a>) -> Self {
        Block::CodeBlock(code_block)
    }
}

impl<'a> From<Link<'a>> for Block<'a> {
    fn from(link: Link<'a>) -> Self {
        Block::Link(link)
    }
}

impl<'a> From<Image<'a>> for Block<'a> {
    fn from(image: Image<'a>) -> Self {
        Block::Image(image)
    }
}

impl<'a> From<OrderedList<'a>> for Block<'a> {
    fn from(ordered_list: OrderedList<'a>) -> Self {
        Block::OrderedList(ordered_list)
    }
}

impl<'a> From<UnorderedList<'a>> for Block<'a> {
    fn from(unordered_list: UnorderedList<'a>) -> Self {
        Block::UnorderedList(unordered_list)
    }
}

impl<'a> From<TaskList<'a>> for Block<'a> {
    fn from(task_list: TaskList<'a>) -> Self {
        Block::TaskList(task_list)
    }
}

impl<'a> From<Footnote<'a>> for Block<'a> {
    fn from(footnote: Footnote<'a>) -> Self {
        Block::Footnote(footnote)
    }
}

impl<'a> From<TextBlock<'a>> for Block<'a> {
    fn from(text_block: TextBlock<'a>) -> Self {
        Block::TextBlock(text_block)
    }
}

impl<'a> From<Newline> for Block<'a> {
    fn from(nl: Newline) -> Self {
        Block::Newline(nl)
    }
}

pub trait ParseIntoBlock<'a>: Parse<'a> {
    fn parse_into_block(input: &'a str) -> IResult<&'a str, Block>;
}

impl<'a, T> ParseIntoBlock<'a> for T
where
    T: Parse<'a> + Into<Block<'a>>,
{
    fn parse_into_block(input: &'a str) -> IResult<&'a str, Block<'a>> {
        let (rest, out) = Self::parse(input)?;
        let out = out.into();
        Ok((rest, out))
    }
}

impl<'a> Block<'a> {
    pub fn parse(input: &'a str) -> IResult<&str, Vec<Self>> {
        let (rest, (blocks, _)) = many_till(
            alt((
                Heading::parse_into_block,
                CodeBlock::parse_into_block,
                Link::parse_into_block,
                Image::parse_into_block,
                Link::parse_into_block,
                Image::parse_into_block,
                OrderedList::parse_into_block,
                UnorderedList::parse_into_block,
                TaskList::parse_into_block,
                Footnote::parse_into_block,
                TextBlock::parse_into_block,
                Newline::parse_into_block,
            )),
            eof,
        )(input)?;

        Ok((rest, blocks))
    }
}

#[cfg(test)]
mod test_parse {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_lines() {
        let input = r#"the quick brown fox
jumps over a lazy dog
"#;

        let (rest, line_1) = parse_line(input).unwrap();
        assert_eq!(line_1, "the quick brown fox");

        let (rest, line_2) = parse_line(rest).unwrap();
        assert_eq!(line_2, "jumps over a lazy dog");

        assert_eq!(rest, "");

        let input = "\ntest";

        let (rest, _) = Newline::parse(input).unwrap();
        assert_eq!(rest, "test");
    }

    #[test]
    fn test_parse_heading() {
        let (_out, heading) = Heading::parse("# h1 \n").unwrap();
        assert_eq!(heading.level, 1);
        assert_eq!(heading.text, "h1");
    }

    #[test]
    fn test_parse_code_block() {
        let input = r#"```
const add = (lhs, rhs) => lhs + rhs;
```
"#;
        let (_, block) = CodeBlock::parse(input).unwrap();
        assert_eq!(block.lang, None);
        assert_eq!(block.contents, "const add = (lhs, rhs) => lhs + rhs;");

        let input = r#"```typescript
const add = (lhs: number, rhs: number): number => lhs + rhs;
```
"#;
        let (_, block) = CodeBlock::parse(input).unwrap();
        assert_eq!(block.lang, Some("typescript"));
        assert_eq!(
            block.contents,
            "const add = (lhs: number, rhs: number): number => lhs + rhs;"
        );
    }

    #[test]
    fn test_parse_link() {
        let input = "[GitHub Pages](https://pages.github.com/)";

        let (_, link) = Link::parse(input).unwrap();

        assert_eq!(link.text, "GitHub Pages");
        assert_eq!(link.url, "https://pages.github.com/");
    }

    #[test]
    fn test_parse_image() {
        let input = "![This is an image](https://myoctocat.com/assets/images/base-octocat.svg)";

        let (_, image) = Image::parse(input).unwrap();

        assert_eq!(image.alt, "This is an image");
        assert_eq!(
            image.source,
            "https://myoctocat.com/assets/images/base-octocat.svg"
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        let input = r#"- George Washington
- John Adams 
- Thomas Jefferson
"#;

        let (_, list) = UnorderedList::parse(input).unwrap();
        assert_eq!(&list[0], "George Washington");
        assert_eq!(&list[1], "John Adams");
        assert_eq!(&list[2], "Thomas Jefferson");
    }

    #[test]
    fn test_parse_ordered_list() {
        let input = r#"1. George Washington
2. John Adams 
3. Thomas Jefferson
"#;

        let (_, list) = OrderedList::parse(input).unwrap();
        assert_eq!(&list[0], "George Washington");
        assert_eq!(&list[1], "John Adams");
        assert_eq!(&list[2], "Thomas Jefferson");
    }

    #[test]
    fn test_parse_tasks() {
        let input = "- [ ] incomplete item\n";

        let (_, task) = Task::parse(input).unwrap();
        assert_eq!(task.completed, false);
        assert_eq!(task.text, "incomplete item");

        let input = "- [x] completed item\n";

        let (_, task) = Task::parse(input).unwrap();
        assert_eq!(task.completed, true);
        assert_eq!(task.text, "completed item");

        let input = r#"- [ ] incomplete item
- [x] completed item
"#;
        let (_, list) = TaskList::parse(input).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_parse_footnote() {
        let input = "[^note]: The note\n";

        let (_, footnote) = Footnote::parse(input).unwrap();
        assert_eq!(footnote.name, "note");
        assert_eq!(footnote.text, vec!["The note"]);

        let input = indoc! {r#"
            [^2]: Every new line should be prefixed with 2 spaces.
              This allows you to have a footnote with multiple lines.
        "#};

        let (_, footnote) = Footnote::parse(input).unwrap();
        assert_eq!(footnote.name, "2");
        assert_eq!(
            footnote.text,
            vec![
                "Every new line should be prefixed with 2 spaces.",
                "This allows you to have a footnote with multiple lines."
            ]
        );
    }

    #[test]
    fn test_parse_text_block() {
        let text = indoc! {"
            the block
            of text

        "};

        let (_, block) = TextBlock::parse(text).unwrap();
        let expected = indoc! {"
            the block
            of text"};
        assert_eq!(block.len(), 1);
        assert_eq!(block.contents[0], TextBlockItem::Text(Text(expected)));

        let text = indoc! {"
            text with [inline](https://google.com) link

        "};

        let (_, block) = TextBlock::parse(text).unwrap();

        assert_eq!(
            block.contents,
            vec![
                TextBlockItem::Text(Text("text with ")),
                TextBlockItem::Link(Link {
                    text: "inline",
                    url: "https://google.com"
                }),
                TextBlockItem::Text(Text(" link"))
            ]
        );
    }

    #[test]
    fn test_parse_block() {
        let input = indoc! {"
            some text
            
            - list
            - list

            [^1]: note
        "};

        let (rest, block) = Block::parse(input).unwrap();

        assert_eq!(rest, "");

        assert_eq!(
            block,
            vec![
                Block::TextBlock(TextBlock {
                    contents: vec![TextBlockItem::Text(Text("some text",),),],
                },),
                Block::UnorderedList(UnorderedList {
                    items: vec!["list", "list",],
                },),
                Block::Newline(Newline,),
                Block::Footnote(Footnote {
                    name: "1",
                    text: vec!["note",],
                },),
            ]
        );
    }
}

