use nom::do_parse;
use nom::many0;
use nom::many1;
use nom::named;
use nom::one_of;
use nom::tag;
use nom::take_until;
use nom::IResult;

use crate::data::Booking;
use chrono::{DateTime, Utc};
use nom::character::complete::digit1;

use chrono::TimeZone;
use chrono_tz::Europe::Berlin;
use nom::Err::Incomplete;

fn parse_int(input: &str) -> IResult<&str, u32> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, u32::from_str_radix(digits, 10).unwrap()))
}

named!(date<&str,DateTime<Utc>>,
    do_parse!(
        day: parse_int >> tag!(".") >> month: parse_int >> tag!(".") >> year: parse_int >> tag!(", ") >>
        hour: parse_int >> tag!(":") >> minute: parse_int >>
        (
            Berlin.ymd(2000 + year as i32, month, day).and_hms(hour, minute, 0).with_timezone(&Utc)
        )
        )
);

//named!(space<&str, Vec<char>>, many0!(char!(' ')));
named!(space<&str, Vec<char>>,
     many1!(one_of!("\t\n\r "))
);

named!(style<&str,()>, do_parse!(
    tag!("style=\"") >> take_until!("\"") >> tag!("\"") >> ()
));

fn parse_description(desc: &str) -> Option<String> {
    htmlescape::decode_html(&desc)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

named!(
    result_entry<&str, Booking>,
    do_parse!(
        take_until!("<TR >") >>
        tag!("<TR >\n") >>

        // id
        space >> tag!("<TD align=right>") >> id: take_until!("</TD>") >> tag!("</TD>\n") >>
        // car
        space >> tag!("<TD nowrap>") >> resource: take_until!("</TD>") >> tag!("</TD>\n") >>
        // user
        space >> tag!("<TD nowrap>") >> user: take_until!("</TD>") >> tag!("</TD>\n") >>

        // start time
        space >> tag!("<TD") >> space >> style >> tag!(">") >> space >> take_until!("<TD") >>
        space >> tag!("<TD nowrap") >> space >> style >> tag!(">") >> space >> start: date >> tag!("</TD>\n") >>

        // end time
        space >> tag!("<TD") >> space >> style >> tag!(">") >> space >> take_until!("<TD") >>
        space >> tag!("<TD nowrap") >> space >> style >> tag!(">") >> space >> end: date >> tag!("</TD>\n") >>

        // duration & details
        space >> take_until!("</TD>") >> tag!("</TD>\n") >>
        space >> take_until!("</TD>") >> tag!("</TD>\n") >>

        // description
        space >> tag!("<TD nowrap>") >> description: take_until!("</TD>") >> tag!("</TD>\n") >>
        space >> tag!("</TR>") >>

        (
            Booking {
                id: id.into(),
                //id: "x".into(),
                resource: resource.into(),
                user: user.into(),
                start: start,
                end: end,
                description: parse_description(description),
            }
        )
    )
);

named!(all_entries<&str, Vec<Booking>>, many0!(result_entry));

pub fn parse_query(body: String) -> Result<Vec<Booking>, ()> {
    println!("Payload: {}", body);

    let result = all_entries(&body);
    println!("Result: {:?}", result);
    Ok(result.unwrap().1)

    /*
        let result = result_entry(&body);
        println!("Result: {:?}", result);
        let (rem, b) = {
            let r = result.unwrap();
            (r.0, r.1)
        };
        let r2 = result_entry(rem);
        println!("Result: {:?}", r2);
    */
    /*
    let result = all_entries(&body).map_err(|e| {
        println!("Err: {:?}", e);
    })?;
     */

    //println!("Rem: {:?}", result.0);
    //Ok(result.1)
    //Ok(vec![b, r2.unwrap().1])
}
