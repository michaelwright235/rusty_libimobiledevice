// jkcoxson
// This one isn't an official tool, but something I think is necessary

use rusty_libimobiledevice::plist_plus2::PString;
use rusty_libimobiledevice::idevice;
use rusty_libimobiledevice::services::instproxy::InstProxyClient;

fn main() {
    const VERSION: &str = "0.1.0";

    env_logger::init();

    let mut udid = "".to_string();
    let mut all = false;

    // Parse arguments
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-u" | "--udid" => {
                udid = args[i + 1].clone();
                i += 1;
            }
            "-h" | "--help" => {
                println!("Usage: idevicelistapps [options]");
                println!();
                println!("Options:");
                println!("  -u, --udid <udid>    : udid of the device to mount");
                println!("  -h, --help           : display this help message");
                println!("  -v, --version        : display version");
                return;
            }
            "-a" | "--all" => {
                all = true;
            }
            "-v" | "--version" => {
                println!("v{}", VERSION);
                return;
            }
            _ => {
                if args[i].starts_with('-') {
                    println!("Unknown flag: {}", args[i]);
                    return;
                }
            }
        }
        i += 1;
    }
    let device = if udid.is_empty() {
        match idevice::get_first_device() {
            Ok(device) => device,
            Err(e) => {
                println!("Error getting devices: {:?}", e);
                return;
            }
        }
    } else {
        match idevice::get_device(udid) {
            Ok(device) => device,
            Err(e) => {
                println!("Error getting devices: {:?}", e);
                return;
            }
        }
    };

    let instproxy_client = match device.new_instproxy_client("idevicelistapps".to_string()) {
        Ok(instproxy) => {
            println!("Successfully started instproxy");
            instproxy
        }
        Err(e) => {
            println!("Error starting instproxy: {:?}", e);
            return;
        }
    };

    let client_opts = InstProxyClient::create_return_attributes(
        vec![("ApplicationType".to_string(), PString::new("Any").into())],
        vec![
            "CFBundleIdentifier".to_string(),
            "CFBundleDisplayName".to_string(),
        ],
    );
    let lookup_results = match instproxy_client.lookup(vec![], Some(client_opts)) {
        Ok(apps) => {
            println!("Successfully looked up apps");
            apps
        }
        Err(e) => {
            println!("Error looking up apps: {:?}", e);
            return;
        }
    };

    for (_, value) in lookup_results.as_dictionary().unwrap() {
        let value_dict = value.as_dictionary().unwrap();
        let id = value_dict
            .get("CFBundleIdentifier")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();
        if id.contains("com.apple") && !all {
            continue;
        }
        println!(
            "{}: {}",
            value_dict
                .get("CFBundleDisplayName")
                .unwrap()
                .as_string()
                .unwrap(),
            id
        );
    }
}
