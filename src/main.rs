use std::env;

const DOMAIN: &str = "https://ssnb.x.moneyforward.com/users/sign_in";

fn main() {
    let args: Vec<String> = env::args().collect();
    let user = &args[1];
    let pass = &args[2];

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
    let total = t.get_inner_text().unwrap();
    println!("{}", total);
}
