use regex::Regex;
const TO_SEARCH: &'static str = "
On 2017-12-31 happy. On 2018-01-01, new Year.
";
use lazy_static::lazy_static;
lazy_static! {
    static ref RE: Regex = Regex::new(r"(?x)
    (?P<year>\d{4}) # the year
    -
    (?P<month>\d{2}) # the month
    -
    (?P<day>\d{2}) # the day
    ").unwrap();
    static ref EMAIL_RE: Regex = Regex::new(r"(?x)
        ^\w+@(?:gmail|163|qq)\.(?:com|cn|com\.cn|net)$
    ").unwrap();
}
fn regex_date(text: &str) -> regex::Captures {
    RE.captures(text).unwrap()
}
fn regex_email(text: &str) -> bool {
    EMAIL_RE.is_match(text)
}


fn main() {
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    for caps in re.captures_iter(TO_SEARCH) {
        println!("year: {}, month: {}, day: {}",
        caps.get(1).unwrap().as_str(),
        caps.get(2).unwrap().as_str(),
        caps.get(3).unwrap().as_str(),);
    }
    println!("Hello, world!");

    let re2 = Regex::new(r"(?x)
    (?P<year>\d{4}) # the year
    -
    (?P<month>\d{2}) # the month
    -
    (?P<day>\d{2}) # the day
    ").unwrap();
    let caps = re2.captures("2018-01-01").unwrap();
    assert_eq!("2018", &caps["year"]);
    assert_eq!("01", &caps["month"]);
    assert_eq!("01", &caps["day"]);
    let after = re2.replace_all("2018-01-01", "$month/$day/$year");
    assert_eq!(after, "01/01/2018");
    let caps = regex_date("2018-01-01");
    assert_eq!("2018", &caps["year"]);
    assert_eq!("01", &caps["month"]);
    assert_eq!("01", &caps["day"]);
    assert_eq!(regex_email("alex@gmail.com"), true);
    assert_eq!(regex_email("alex@gmail.cn.com"), false);
}
