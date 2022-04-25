use nom::IResult;

pub mod markdown;
pub mod frontmatter;

pub trait Parse<'a>: Sized {
    fn parse(input: &'a str) -> IResult<&str, Self>;
}
