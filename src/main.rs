use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

const DOMAIN: &str = "https://ssnb.x.moneyforward.com/users/sign_in";

// 資産データを保持する構造体
#[derive(Default,Debug)]
struct Portfolio {
    total: f64,
    money: f64,
    treasury: f64,
    mmf: f64,
    reit: f64,
    us_treasury_map: HashMap<i32, i64>,
}

impl Portfolio {
    fn stock(&self) -> f64 {
        self.total - self.money - self.treasury - self.reit - self.mmf
    }

    fn print_ratios(&self) {
        let total = self.total;
        println!("\n## 比率");
        let list = [
            ("株式", self.stock()),
            ("債券", self.treasury),
            ("REIT", self.reit),
            (" MMF", self.mmf),
            ("現金", self.money),
        ];
        for (label, val) in list {
            println!("{}: {:.2}%", label, val / total * 100.0);
        }

        if !self.us_treasury_map.is_empty() {
            let us_total: i64 = self.us_treasury_map.values().sum();
            println!("(米国債: {:.2}%)", (us_total as f64) / total * 100.0);
        }
    }
}

// ヘルパー関数: 要素からテキストを取得してカンマを除去
fn get_clean_text(tab: &headless_chrome::browser::Tab, selector: &str) -> String {
    tab.wait_for_element(selector)
        .map(|e| e.get_inner_text().unwrap().replace(',', ""))
        .unwrap()
}

// ヘルパー関数: テキストから正規表現で最初の数値を抽出
fn extract_f64(text: &str, pattern: &str) -> f64 {
    Regex::new(pattern)
        .unwrap()
        .captures(text)
        .and_then(|c| c.get(1))
        .map_or(0.0, |m| m.as_str().parse().unwrap())
}

fn show(tab: Arc<headless_chrome::browser::Tab>) {
    tab.wait_for_element("li.global-menu-item:nth-child(4) > a").unwrap().click().unwrap();

    let mut p = Portfolio::default();

    // 1. 資産総額
    let total_text = get_clean_text(&tab, ".heading-radius-box");
    p.total = extract_f64(&total_text, r"(\d+)円");
    println!("## 資産総額\n{}円", p.total);

    // 2. 現金と債券
    let breakdown_text = get_clean_text(&tab, "table.table:nth-child(4)");
    p.money = extract_f64(&breakdown_text, r"現金.+\s+(\d+)円");
    p.treasury = extract_f64(&breakdown_text, r"債券\s+(\d+)円");

    // 3. 投資信託の振り分け
    let mf_text = get_clean_text(&tab, ".table-mf");
    let re_mmf = Regex::new(r"マネー・マーケット・ファンド.+\s+(\d+)円\s+[-\d]+円\s+[-\d]+円").unwrap();
    let re_treasury = Regex::new(r"債券.+\s+(\d+)円\s+[-\d]+円\s+[-\d]+円").unwrap();
    let re_reit = Regex::new(r"リート.+\s+(\d+)円\s+[-\d]+円\s+[-\d]+円").unwrap();
    for line in mf_text.lines() {
        if let Some(c) = re_mmf.captures(line) {
            p.mmf += c[1].parse::<f64>().unwrap();
        } else if let Some(c) = re_treasury.captures(line) {
            p.treasury += c[1].parse::<f64>().unwrap();
        } else if let Some(c) = re_reit.captures(line) {
            p.reit += c[1].parse::<f64>().unwrap();
        }
    }

    // 4. 米国債（満期別）
    if let Ok(e) = tab.wait_for_element(".table-bd") {
        let bd_text = e.get_inner_text().unwrap().replace(',', "");
        let re_us = Regex::new(r"米国国債.+\s+(\d{4})/\d+/\d+満期\s+(\d+)円").unwrap();
        for line in bd_text.lines() {
            if let Some(c) = re_us.captures(line) {
                let year: i32 = c[1].parse().unwrap();
                let amount: i64 = c[2].parse().unwrap();
                *p.us_treasury_map.entry(year).or_insert(0) += amount;
            }
        }
    }

    // 結果表示
    p.print_ratios();

    if !p.us_treasury_map.is_empty() {
        println!("\n## 米国債満期");
        let mut years: Vec<_> = p.us_treasury_map.iter().collect();
        years.sort_by_key(|k| k.0);
        for (y, amt) in years {
            println!("{}年 {}円", y, amt);
        }
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
