extern crate nom;

use std::collections::HashSet;
use std::fs;
use std::io;
use std::iter::FromIterator;

use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till, take_while_m_n},
    character::complete::{digit1, multispace0},
    combinator::{map_res, not, opt},
    sequence::{preceded, terminated},
    IResult
};

fn main() {
    let input = fs::read_to_string("input.txt").expect("Dead file");
    let count = input.split("\n\n").filter(|b| check(b)).count();

    println!("Valid passports: {:?}", count);

    let mut deep_count = 0;

    for l in input.split("\n\n") {
        let c = check_contents(l);

        if c.is_ok() {
            deep_count += 1;
        }
    }

    println!("Valid content passports: {}", deep_count);
}

fn check(block: &str) -> bool {
    let required_fields: HashSet<&str> =
        HashSet::from_iter(vec!["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"]);

    let present_fields: HashSet<&str> = HashSet::from_iter(
        block
            .split_whitespace()
            .map(|f| f.split(':').next().unwrap()),
    );

    let diff = required_fields.difference(&present_fields);

    diff.copied().next().is_none()
}

fn check_contents(input: &str) -> IResult<&str, (&str, &str, u16, u16, u16, &str, u16, Option<&str>)> {
    permutation((
        check_eye_color,
        check_passport_id,
        check_birth_year,
        check_issue_year,
        check_expiry_year,
        check_hair_color,
        check_height,
        check_cid,
    ))(input)
}

#[test]
fn test_check_contents() {
    assert_eq!(
        check_contents("hgt:182cm pid:123456789 ecl:amb hcl:#000000 byr:1981 iyr:2020 cid:12938981 eyr:2020"),
        Ok(("", ("amb", "123456789", 1981, 2020, 2020, "000000", 182, Some("12938981"))))
    );

    assert_eq!(
        check_contents(
            r"iyr:2011
        pid:123456789 ecl:amb
        byr:1981 hgt:60in eyr:2025 hcl:#ffffff"
        ),
        Ok(("", ("amb", "123456789", 1981, 2011, 2025, "ffffff", 60, None)))
    );

    assert_eq!(check_contents(r"pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
    hcl:#623a2f"), Ok(("", ("grn", "087499704", 1980, 2012, 2030, "623a2f", 74, None))));
    assert_eq!(check_contents(r"eyr:2029 ecl:blu cid:129 byr:1989
    iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm"), Ok(("", ("blu", "896056539", 1989, 2014, 2029, "a97842", 165, Some("129")))));
    assert_eq!(check_contents(r"hcl:#888785
    hgt:164cm byr:2001 iyr:2015 cid:88
    pid:545766238 ecl:hzl
    eyr:2022"), Ok(("", ("hzl", "545766238", 2001, 2015, 2022, "888785", 164, Some("88")))));
    assert_eq!(check_contents(r"iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719"), Ok(("", ("blu", "093154719", 1944, 2010, 2021, "b6652a", 158, None))));

    assert!(check_contents(r"eyr:1972 cid:100
    hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926").is_err());
    assert!(check_contents(r"iyr:2019
    hcl:#602927 eyr:1967 hgt:170cm
    ecl:grn pid:012533040 byr:1946").is_err());
    assert!(check_contents(r"hcl:dab227 iyr:2012
    ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277").is_err());
    assert!(check_contents(r"hgt:59cm ecl:zzz
    eyr:2038 hcl:74454a iyr:2023
    pid:3556412378 byr:2007").is_err());
}

fn check_birth_year(input: &str) -> IResult<&str, u16> {
    check_year_record(input, "byr", 1920, 2002)
}

fn check_issue_year(input: &str) -> IResult<&str, u16> {
    check_year_record(input, "iyr", 2010, 2020)
}

fn check_expiry_year(input: &str) -> IResult<&str, u16> {
    check_year_record(input, "eyr", 2020, 2030)
}

fn check_year_record<'a>(
    input: &'a str,
    field: &str,
    lower_bound: u16,
    upper_bound: u16,
) -> IResult<&'a str, u16> {
    terminated(
        preceded(
            tag((field.to_string() + ":").as_str()),
            map_res(digit1, |s: &str| match s.parse::<u16>() {
                Ok(n) if n < lower_bound || n > upper_bound => {
                    Err(io::Error::new(io::ErrorKind::Other, "out of range"))
                }
                Ok(n) => Ok(n),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
            }),
        ),
        multispace0,
    )(input)
}

#[test]
fn test_check_year_record() {
    assert_eq!(
        check_year_record("byr:1981", "byr", 1920, 2002),
        Ok(("", 1981))
    );

    assert!(check_year_record("byr:1900", "byr", 1920, 2002).is_err());
}

fn check_eye_color(input: &str) -> IResult<&str, &str> {
    let color = alt((
        tag("amb"),
        tag("blu"),
        tag("brn"),
        tag("gry"),
        tag("grn"),
        tag("hzl"),
        tag("oth"),
    ));

    terminated(preceded(tag("ecl:"), color), multispace0)(input)
}

#[test]
fn test_check_eye_color() {
    assert_eq!(check_eye_color("ecl:amb"), Ok(("", "amb")));

    assert!(check_eye_color("  ecl:amb").is_err());

    assert_eq!(check_eye_color("ecl:amb  "), Ok(("", "amb")));

    assert!(check_eye_color("ecl:bul").is_err());
}

fn check_cid(input: &str) -> IResult<&str, Option<&str>> {
    opt(terminated(preceded(tag("cid:"), take_till(|c| c == ' ' || c == '\n')), multispace0))(input)
}

#[test]
fn test_check_cid() {
    assert_eq!(check_cid("cid:123alper  lbabla"), Ok(("lbabla", Some("123alper"))));
    assert_eq!(check_cid("eid:123alper  lbabla"), Ok(("eid:123alper  lbabla", None)));
    assert_eq!(check_cid("   cid:123alper  lbabla"), Ok(("   cid:123alper  lbabla", None)));
    assert_eq!(check_cid("cid:alper"), Ok(("", Some("alper"))));
    assert_eq!(check_cid("cid:123"), Ok(("", Some("123"))));
}

fn check_passport_id(input: &str) -> IResult<&str, &str> {
    let pid = take_while_m_n(9, 9, |c: char| c.is_digit(10));

    terminated(
        terminated(preceded(tag("pid:"), pid), not(digit1)),
        multispace0,
    )(input)
}

#[test]
fn test_check_passport_id() {
    assert_eq!(check_passport_id("pid:123456789"), Ok(("", "123456789")));
    assert_eq!(
        check_passport_id("pid:123456789    "),
        Ok(("", "123456789"))
    );
    assert!(check_passport_id("pid:12345678912    ").is_err());
}

fn check_hair_color(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("hcl:#")(input)?;

    terminated(take_while_m_n(6, 6, |c: char| c.is_digit(16)), multispace0)(input)
}

#[test]
fn test_check_hair_color() {
    assert_eq!(check_hair_color("hcl:#ffffff"), Ok(("", "ffffff")));
    assert_eq!(check_hair_color("hcl:#000000  "), Ok(("", "000000")));

    assert!(check_hair_color("hcl:#11111").is_err());
    assert!(check_hair_color("hcl:5d90f0").is_err());
}

fn check_height(input: &str) -> IResult<&str, u16> {
    let (input, _) = tag("hgt:")(input)?;

    terminated(
        alt((
            terminated(
                map_res(digit1, |s: &str| match s.parse::<u16>() {
                    Ok(n) if n < 150 || n > 193 => {
                        Err(io::Error::new(io::ErrorKind::Other, "out of range"))
                    }
                    Ok(n) => Ok(n),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                }),
                tag("cm"),
            ),
            terminated(
                map_res(digit1, |s: &str| match s.parse::<u16>() {
                    Ok(n) if n < 59 || n > 76 => {
                        Err(io::Error::new(io::ErrorKind::Other, "out of range"))
                    }
                    Ok(n) => Ok(n),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                }),
                tag("in"),
            ),
        )),
        multispace0,
    )(input)
}
