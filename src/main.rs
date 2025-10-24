use std::env;
use std::sync::Arc;
use std::collections::HashMap;
use regex::Regex;

const DOMAIN: &str = "https://ssnb.x.moneyforward.com/users/sign_in";

fn show(tab: Arc<headless_chrome::browser::Tab>) {
    let a1 = tab.wait_for_element("li.global-menu-item:nth-child(4) > a:nth-child(1)").unwrap();
    a1.click().unwrap();
    
    let t = tab.wait_for_element(".heading-radius-box").unwrap();
    let mut total = t.get_inner_text().unwrap();
    total.retain(|c| c != ',');
    let re = Regex::new(r"\s+(\d+)円").unwrap();
    let caps_total = re.captures(&total).unwrap();
    let total_f: f64 = caps_total[1].parse().unwrap();
    println!("## 資産総額");
    println!("{}円", total_f);

    let t1 = tab.wait_for_element("table.table:nth-child(4)").unwrap();
    let mut breakdown = t1.get_inner_text().unwrap();
    breakdown.retain(|c| c != ',');
    let re_money = Regex::new(r"現金.+\s+(\d+)円").unwrap();
    let caps_money = re_money.captures(&breakdown).unwrap();
    let money_f: f64 = caps_money[1].parse().unwrap();
    let re_treasury = Regex::new(r"債券\s+(\d+)円").unwrap();
    let caps_treasury = re_treasury.captures(&breakdown).unwrap();
    let treasury_f: f64 = caps_treasury[1].parse().unwrap();

    let t3 = tab.wait_for_element(".table-mf").unwrap();
    let mut mutualfund = t3.get_inner_text().unwrap();
    mutualfund.retain(|c| c != ',');
    let mut mmf_f = 0.0;
    let re_mmf = Regex::new(r"マネー・マーケット・ファンド.+\s+(\d+)円\s+\d+円\s+\d+円").unwrap();
    for mline in mutualfund.lines() {
        let caps_mmf = re_mmf.captures(mline);
        match caps_mmf {
            None => continue,
            Some(c) => {
                mmf_f = c[1].parse().unwrap();
            }
        }
    }

    let t2 = tab.wait_for_element(".table-bd").unwrap();
    let mut treasury = t2.get_inner_text().unwrap();
    treasury.retain(|c| c != ',');
    let mut map = HashMap::new();
    let re_us_treasury = Regex::new(r"米国国債.+\s+(\d\d\d\d)/\d+/\d+満期\s+(\d+)円").unwrap();
    for tline in treasury.lines() {
        let caps_us_treasury = re_us_treasury.captures(tline);
        match caps_us_treasury {
            None => continue,
            Some(c) => {
                let year: i32 = c[1].parse().unwrap();
                let m: i64 = c[2].parse().unwrap();
                let count = map.entry(year).or_insert(0);
                *count += m;
            }
        }
    }
    let mut v: Vec<_> = map.into_iter().collect();
    let mut treasury_us_f = 0.0;
    for i in &v {
        treasury_us_f += i.1 as f64;
    }

    let stock_f = total_f - money_f - treasury_f;
    println!("\n## 比率");
    println!("株式: {}%", (stock_f - mmf_f) / total_f * 100.0);
    println!("債券: {}%", treasury_f / total_f * 100.0);
    println!("  日本国債: {}%", (treasury_f - treasury_us_f) / total_f * 100.0);
    println!("    米国債: {}%", treasury_us_f / total_f * 100.0);
    println!(" MMF: {}%", mmf_f / total_f * 100.0);
    println!("現金: {}%", money_f / total_f * 100.0);

    println!("\n## 米国国債満期");
    v.sort();
    for i in &v {
        println!("{}年 {}円", i.0, i.1);
    }
}

fn sync(tab: Arc<headless_chrome::browser::Tab>) {
    let a1 = tab.wait_for_element("li.global-menu-item:nth-child(6) > a:nth-child(1)").unwrap();
    a1.click().unwrap();

    let inputs = tab.wait_for_elements("#account-table > tbody:nth-child(1) > tr > td > form > input").unwrap();
    let mut n = 0;
    for e in inputs.iter() {
        let v = e.get_attribute_value("value").unwrap().unwrap();
        if v == "更新" {
            e.click().unwrap();
            n += 1;
            eprint!("{}.", n);
        }
    }
    eprintln!();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let user = &args[2];
    let pass = &args[3];

    let browser = headless_chrome::Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    tab.navigate_to(DOMAIN).unwrap();

    tab.wait_for_element("#sign_in_session_service_email").unwrap().click().unwrap();
    tab.type_str(user).unwrap();
    tab.wait_for_element("#sign_in_session_service_password").unwrap().click().unwrap();
    tab.type_str(pass).unwrap();
    tab.press_key("Enter").unwrap();

    if args[1] == "show" {
        show(tab);
    } else if args[1] == "sync" {
        sync(tab);
    }
}
