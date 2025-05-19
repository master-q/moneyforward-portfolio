use std::env;
use regex::Regex;

const DOMAIN: &str = "https://ssnb.x.moneyforward.com/users/sign_in";

fn main() {
    let args: Vec<String> = env::args().collect();
    let user = &args[2];
    let pass = &args[3];

    let browser = headless_chrome::Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    tab.navigate_to(&(DOMAIN.to_string())).unwrap();

    tab.wait_for_element("#sign_in_session_service_email").unwrap().click().unwrap();
    tab.type_str(&user).unwrap();
    tab.wait_for_element("#sign_in_session_service_password").unwrap().click().unwrap();
    tab.type_str(&pass).unwrap();
    tab.press_key("Enter").unwrap();

    let a1 = tab.wait_for_element("li.global-menu-item:nth-child(4) > a:nth-child(1)").unwrap();
    a1.click().unwrap();
    
    let t = tab.wait_for_element(".heading-radius-box").unwrap();
    let mut total = t.get_inner_text().unwrap();
    total.retain(|c| c != ',');
    let re = Regex::new(r"\s+(\d+)円").unwrap();
    let caps_total = re.captures(&total).unwrap();
    let total_f: f64 = caps_total[1].parse().unwrap();

    let t1 = tab.wait_for_element("table.table:nth-child(4)").unwrap();
    let mut breakdown = t1.get_inner_text().unwrap();
    breakdown.retain(|c| c != ',');
    let re_money = Regex::new(r"現金.+\s+(\d+)円").unwrap();
    let caps_money = re_money.captures(&breakdown).unwrap();
    let money_f: f64 = caps_money[1].parse().unwrap();
    let re_treasury = Regex::new(r"債券\s+(\d+)円").unwrap();
    let caps_treasury = re_treasury.captures(&breakdown).unwrap();
    let treasury_f: f64 = caps_treasury[1].parse().unwrap();

    let stock_f = total_f - money_f - treasury_f;
    println!("株式: {}%", stock_f / total_f * 100.0);
    println!("債券: {}%", treasury_f / total_f * 100.0);
    println!("現金: {}%", money_f / total_f * 100.0);

    let t2 = tab.wait_for_element(".table-bd").unwrap();
    let mut treasury = t2.get_inner_text().unwrap();
    treasury.retain(|c| c != ',');
    for tline in treasury.lines() {
        let re_us_treasury = Regex::new(r"米国国債.+\s+(\d\d\d\d)/\d+/\d+満期\s+(\d+)円").unwrap();
        let caps_us_treasury = re_us_treasury.captures(&tline);
        match caps_us_treasury {
            None => continue,
            Some(c) => {
                println!("{}年 {}円", &c[1], &c[2])
            }
        }
    }
}
