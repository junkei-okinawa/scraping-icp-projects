use dotenv::dotenv;
use env_logger;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Dapp {
    logo_url: String,
    project_name: String,
    data_social: Vec<Social>,
    category_list: Vec<String>,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Social {
    name: String,
    url: String,
}

pub fn json_to_social(json: &str) -> Result<Social, serde_json::Error> {
    serde_json::from_str(json)
}

fn get_innerhtml_text(elm: &headless_chrome::Element, cunction_declaration: &str) -> String {
    let elm_result = elm.call_js_fn(cunction_declaration, vec![], true);
    let result = match elm_result {
        Ok(text) => {
            if text.value.is_none() {
                String::from("")
            } else {
                text.value.unwrap().as_str().unwrap().to_string()
            }
        }
        _ => String::from(""),
    };
    result
}

macro_rules! get_innerhtml_text {
    ($elm: expr, $cunction_declaration: expr) => {
        get_innerhtml_text($elm, $cunction_declaration)
    };
    ($elm: expr) => {
        get_innerhtml_text($elm, "function() { return this.textContent;}")
    };
}

fn get_logo_url(elm: &headless_chrome::Element) -> String {
    let elm_result = elm.call_js_fn("function() { return this.src;}", vec![], true);
    let result = match elm_result {
        Ok(text) => {
            if text.value.is_none() {
                String::from("")
            } else {
                text.value.unwrap().as_str().unwrap().to_string()
            }
        }
        _ => String::from(""),
    };
    result
}

fn get_data_social(elm: &headless_chrome::Element) -> Vec<Social> {
    let mut social_list: Vec<Social> = Vec::new();
    let data_elm_list_result = elm.find_elements("li>a");
    match data_elm_list_result {
        Ok(data_elm_list) => {
            for (i, data_elm) in data_elm_list.iter().enumerate() {
                let elm_result = data_elm.call_js_fn(
                    r#"function() {
                        var data = {};
                        data["name"] = this.innerText;
                        data["url"] = this.href;
                        return JSON.stringify(data);
                    }"#,
                    vec![],
                    true,
                );

                match elm_result {
                    Ok(text) => {
                        if text.value.is_none() == false {
                            let json_result = json_to_social(text.value.unwrap().as_str().unwrap());
                            match json_result {
                                Ok(social) => social_list.insert(i, social),
                                Err(e) => debug!("json_to_social error: {}", e),
                            }
                        }
                    }
                    Err(e) => warn!("data_elm call_js_fn error: {}", e),
                };
            }
        }
        Err(e) => warn!("data_elm_list_result find error: {}", e),
    }
    social_list
}

fn get_category_list(elm: &headless_chrome::Element) -> Vec<String> {
    let mut category_list: Vec<String> = vec![];
    let data_elm_list_result = elm.find_elements("li");
    match data_elm_list_result {
        Ok(data_elm_list) => {
            for (i, data_elm) in data_elm_list.iter().enumerate() {
                let elm_result =
                    data_elm.call_js_fn("function() { return this.textContent;}", vec![], true);

                match elm_result {
                    Ok(text) => {
                        if text.value.is_none() == false {
                            category_list
                                .insert(i, text.value.unwrap().as_str().unwrap().to_string())
                        }
                    }
                    Err(e) => warn!("data_elm call_js_fn error: {}", e),
                };
            }
        }
        Err(e) => warn!("data_elm_list_result find error: {}", e),
    }
    category_list
}

fn write_file(dapps: &HashMap<usize, Dapp>, dapps_json_path: &str) -> std::io::Result<()> {
    // serialized & indented
    let serialized: String = serde_json::to_string_pretty(&dapps).unwrap();

    // write
    let mut file = File::create(dapps_json_path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // >>> scraping base settings
    dotenv().ok();
    env_logger::init();
    let ENV = dotenv::var("ENVIRONMENT").expect("ENVIRONMENT must be set");
    let dapps_json_path = &env::var("DAPPS_JSON_PATH").expect("DAPPS_JSON_PATH must be set");
    let loading_data = dotenv::var("LOADING_DATA").expect("LOADING_DATA must be set");
    let duration_default_timeout: u64 = dotenv::var("DURATION_DEFAULT_TIMEOUT")
        .expect("DURATION_DEFAULT_TIMEOUT must be set")
        .parse()
        .unwrap();
    let duration_loading_contents: u64 = dotenv::var("DURATION_LOADONG_CONTENTS")
        .expect("DURATION_LOADONG_CONTENTS must be set")
        .parse()
        .unwrap();
    let duration_loading_navigated_page: u64 = dotenv::var("DURATION_LOADONG_NAVIGATED_PAGE")
        .expect("DURATION_LOADONG_NAVIGATED_PAGE must be set")
        .parse()
        .unwrap();
    let duration_custom_timeout: u64 = dotenv::var("DURATION_CUSTOM_TIMEOUT")
        .expect("DURATION_CUSTOM_TIMEOUT must be set")
        .parse()
        .unwrap();
    let base_url = &dotenv::var("BASE_URL").expect("BASE_URL must be set");
    let data_load_button_xpath =
        dotenv::var("DATA_LOAD_BUTTON").expect("DATA_LOAD_BUTTON must be set");
    let dapps_list_xpath = &dotenv::var("DAPPS_LIST").expect("DAPPS_LIST must be set");
    let h3_elm_function_str = &dotenv::var("H3_ELM_FUNCTION").expect("H3_ELM_FUNCTION must be set");
    let logo_url_xpath = &dotenv::var("LOGO_URL").expect("LOGO_URL must be set");
    let project_name_xpath = &dotenv::var("PROJECT_NAME").expect("PROJECT_NAME must be set");
    let data_social_xpath = &dotenv::var("DATA_SOCIAL").expect("DATA_SOCIAL must be set");
    let category_list_xpath = &dotenv::var("CATEGORY_LIST").expect("CATEGORY_LIST must be set");
    let description_xpath = &dotenv::var("DESCRIPTION").expect("DESCRIPTION must be set");
    let scrape_xpath_list = vec![
        &logo_url_xpath,
        &data_social_xpath,
        &category_list_xpath,
        &description_xpath,
    ];

    macro_rules! project_name_get_xpath {
        () => {
            "//h3[text()='{}')]"
        };
    }
    macro_rules! project_name_retry_xpath {
        () => {
            "//h3[contains(text(),'{}')]"
        };
    }
    // <<< scraping base settings

    // >>> browser & tab settings
    // 1920x1080のウィンドウサイズでブラウザを立ち上げ
    let options = LaunchOptionsBuilder::default()
        .window_size(Some((1920, 1080)))
        .build()
        .expect("Fail to build");
    let browser = Browser::new(options)?;

    // rust-headless-chromeのページに遷移
    let tab = browser.wait_for_initial_tab()?;
    tab.set_default_timeout(Duration::from_secs(duration_default_timeout));
    // <<< browser & tab settings

    // >>> base page settings
    let base_url = base_url;
    tab.navigate_to(&base_url)?;
    tab.wait_until_navigated()?;

    // >>> loading all project elements
    if loading_data.parse().unwrap() {
        let mut counter = 0;
        loop {
            sleep(Duration::from_secs(duration_loading_contents));
            debug!("contents loading counter: {}", counter);
            // run scrolle page end js method.
            tab.evaluate(
                r#"
                                var element = document.documentElement;
                                var bottom = element.scrollHeight - element.clientHeight;
                                window.scroll(0, bottom);
                            "#,
                true,
            )?;
            let wait_result = tab.wait_for_xpath_with_custom_timeout(
                &data_load_button_xpath,
                Duration::from_secs(duration_custom_timeout),
            );
            match wait_result {
                Ok(loading_button) => {
                    loading_button.click()?;
                    counter += 1;
                }
                Err(_) => break,
            }
        }
    }
    // <<< loading all project elements
    // <<< base page settings

    // >>> get dapps project names
    sleep(Duration::from_secs(duration_loading_contents));
    if ENV != "prod" {
        let png =
            tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
        std::fs::write("./base_page.png", png)?;
    }
    let elements = tab.find_elements(dapps_list_xpath)?;

    let mut dapps: HashMap<usize, Dapp> = HashMap::new();

    for (index, element) in elements.iter().enumerate() {
        let h3_text = get_innerhtml_text!(&element, h3_elm_function_str);

        if h3_text != String::from("") {
            let dapp: Dapp = Dapp {
                logo_url: String::from(""),
                project_name: h3_text,
                data_social: Vec::new(),
                category_list: Vec::new(),
                description: String::from(""),
            };
            dapps.insert(index.clone(), dapp);
        }
    }
    // <<< get dapps project names

    debug!("dapps.len(): {}", dapps.len());

    // >>> get dapps project data
    for (index, dapp) in &mut dapps {
        debug!("================\n");
        debug!("{:?}", dapp.project_name);

        sleep(Duration::from_secs(duration_loading_navigated_page));
        let xpath = format!(project_name_get_xpath!(), &dapp.project_name);
        debug!("{}", xpath);
        let h3_elm_result = tab.wait_for_xpath(&xpath);
        let h3_elm_result = match h3_elm_result {
            Err(_) => {
                // project_nameの末尾の空白が自動的に欠落しているの、部分一致検索に変更してリトライ
                let xpath = format!(project_name_retry_xpath!(), &dapp.project_name);
                debug!("retry xpath: {}", xpath);
                tab.wait_for_xpath(&xpath)
            }
            Ok(h3_elm) => Ok(h3_elm),
        };

        match h3_elm_result {
            Ok(h3_elm) => {
                h3_elm.click()?;
                debug!("after click");
                sleep(Duration::from_secs(duration_loading_contents));
                let _ = tab.wait_for_xpath_with_custom_timeout(
                    &xpath,
                    Duration::from_secs(duration_custom_timeout),
                );
                sleep(Duration::from_secs(duration_loading_contents));
                // i = 0: logo_url, 1: data_social, 2: category_list, 3: description
                for (i, xpath) in scrape_xpath_list.iter().enumerate() {
                    let wait_result = tab.wait_for_xpath_with_custom_timeout(
                        &xpath,
                        Duration::from_secs(duration_custom_timeout),
                    );
                    match wait_result {
                        Ok(elm) => {
                            match i {
                                0 => {
                                    dapp.logo_url = get_logo_url(&elm);
                                }
                                1 => {
                                    dapp.data_social = get_data_social(&elm);
                                }
                                2 => {
                                    dapp.category_list = get_category_list(&elm);
                                }
                                3 => {
                                    dapp.description = get_innerhtml_text!(&elm);
                                }
                                _ => warn!("not covered i: {}", i),
                            };
                        }
                        Err(_) => break,
                    }
                }
                if ENV != "prod" {
                    // PNGでキャプチャを撮影してファイルに保存
                    let png = tab.capture_screenshot(
                        Page::CaptureScreenshotFormatOption::Png,
                        None,
                        None,
                        true,
                    )?;
                    std::fs::write(
                        format!("./images/{}_{}.png", index, &dapp.project_name),
                        png,
                    )?;
                }
                info!("scraped data: {:?}", dapp);
                tab.navigate_to(&base_url)?;
            }
            Err(error) => warn!("h3_elm faild: {}", error),
        }
    }
    // <<< get dapps project data

    let write_file_result = write_file(&dapps, &dapps_json_path);
    match write_file_result {
        Err(e) => panic!("{}", e),
        _ => debug!("finish all dapps project export json."),
    }
    Ok(())
}
