use nom::complete;
use nom::do_parse;
use nom::many0;
use nom::many_till;
use nom::named;
use nom::one_of;
use nom::opt;
use nom::tag;
use nom::take_until;
use nom::IResult;

use anyhow::anyhow;

use crate::data::Booking;
use chrono::{DateTime, Utc};
use nom::character::complete::digit1;

use chrono::TimeZone;
use chrono_tz::Europe::Berlin;

fn parse_int(input: &str) -> IResult<&str, u32> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, u32::from_str_radix(digits, 10).unwrap()))
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, usize::from_str_radix(digits, 10).unwrap()))
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

named!(space<&str, Vec<char>>,
     many0!(one_of!("\t\n\r "))
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

#[derive(Copy, Clone, Debug)]
pub struct Paging {
    pub from: usize,
    pub to: usize,
    pub total: usize,
}

named!(
    paging<&str,Paging>,
    do_parse!(
        take_until!("<B>Eintr&auml;ge") >> tag!("<B>Eintr&auml;ge") >>
        space >> from: parse_usize >>
        space >> tag!("bis") >> space >> to: parse_usize >>
        space >> tag!("von") >> space >> total: parse_usize >> tag!("<BR>") >>
        (
            Paging{
                from, to, total,
            }
        )
    )
);

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
        space >> tag!("</TR>") >> space >>

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

// named!(all_entries<&str, Vec<Booking>>, many0!(result_entry));

named!(all_entries<&str, (Option<Paging>,Vec<Booking>)>,
    do_parse!(
        paging: opt!(complete!(paging)) >>
        entries: many_till!(result_entry, tag!("</TABLE>")) >>
        (
            (paging, entries.0)
        )
    )
);

#[derive(Clone, Debug)]
pub struct ListResponse {
    pub paging: Option<Paging>,
    pub bookings: Vec<Booking>,
}

pub fn parse_query(body: &String) -> anyhow::Result<ListResponse> {
    log::trace!("Payload: {}", body);

    if body.contains("<B>Die Suche ergab keine Treffer!</B>") {
        return Ok(ListResponse {
            paging: None,
            bookings: vec![],
        });
    }

    match all_entries(&body) {
        Ok(result) => {
            // the let isn't necessary, but works around a parser issue of IntelliJ and rustfmt
            // when using "result.1.0", where "1.0" is interpreted as an float, rather than two
            // identifiers
            let r = result.1;
            Ok(ListResponse {
                paging: r.0,
                bookings: r.1,
            })
        }
        e => {
            log::debug!("Payload: {}", body);
            log::debug!("Parse failure: {:?}", e);

            //Err(e.context("Failed to parse"))
            Err(anyhow!("Failed to parse"))
        }
    }
}
