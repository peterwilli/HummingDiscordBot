use regex::Regex;

pub fn beautify_bot_name(bot_name: &str) -> String {
    let re = Regex::new(r"hummingbot-(.+)-").unwrap();
    let caps = re.captures(bot_name).unwrap();
    let middle_name = caps.get(1).unwrap().as_str().to_string();
    middle_name
}
