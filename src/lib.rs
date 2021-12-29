use std::str::FromStr;
use std::error::Error;
use std::time::Duration;
use bluer::{Session, Adapter, Address};
use tokio::time::timeout;
use pam::module::{PamHandle, PamHooks};
use pam::constants::{PamResultCode, PamFlag};
use std::ffi::CStr;


#[macro_use] extern crate pam;

mod settings;

#[derive(Debug)]
struct ScanError {
    pub msg: String
}
impl ScanError {
    pub fn with_msg(s: &str) -> Box<Self> {
        Box::new(ScanError {
            msg: s.to_string()
        })
    }
}
impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>{
        write!(f, "Error while scanning for device: {}", self.msg)
    }
}
impl std::error::Error for ScanError {}

async fn is_device_in_range(adapter: &Adapter, target_address: Address) -> Result<bool, Box<dyn Error>> {
    let device = adapter.device(target_address)?;
    if device.is_connected().await? {
        println!("Device in range");
        return Ok(true);
    }
    println!("Device not connected");
    if let Ok(_) = device.connect().await {
        println!("Successfully connected device");
        return Ok(true);
    }
    return Ok(false);
}

async fn scan_for_target(adapter: &Adapter) -> Result<(), Box<dyn Error>> {
    let s = settings::get();
    let target_address: Address = Address::from_str(&s.target)?;
    let interval = Duration::from_millis((s.interval.unwrap_or(0.5) * 1000.) as u64);
    println!("Looking for: {:?}", target_address);

    let known = adapter.device_addresses().await?;

    if !known.contains(&target_address) {
        return Err(ScanError::with_msg("Device not paired"));
    }
    loop {
        if is_device_in_range(adapter, target_address).await? {
            return Ok(());
        }
        std::thread::sleep(interval);
    }
}

async fn init_adapter() -> Result<Adapter, Box<dyn Error>> {
    let s = settings::get();

    let session = Session::new().await?;
    let found_adapters = session.adapter_names().await?;
    let adapter;
    if s.adapter.is_some() && found_adapters.contains(&s.adapter.as_ref().unwrap()) {
        adapter = session.adapter(&s.adapter.as_ref().unwrap())?;
    } else {
        adapter = session.adapter(&found_adapters[0])?;
    }

    println!("Found adapters: {:?}", found_adapters);
    println!("Using adapter: {:?}", adapter);

    if !adapter.is_powered().await? {
        adapter.set_powered(true).await?;
        println!("Powered on adapter");
    } else {
        println!("Adapter is powered on");
    }

    Ok(adapter)
}

pub async fn run(settings_file_path: Option<String>) -> Result<(), Box<dyn Error>> {
    settings::load_settings(&settings_file_path.unwrap_or("./settings.yml".to_string()))?;
    let s = settings::get();
    let scan_timeout = Duration::from_secs(s.scan_timeout.unwrap_or(5));

    let adapter = init_adapter().await?;
    let scan_task = scan_for_target(&adapter);
    if s.scan_timeout.is_some() && s.scan_timeout.unwrap() == 0 {
        println!("WARNING: scan timeout is set to unlimited, this is only good for testing!");
        return scan_task.await;
    }
    return timeout(scan_timeout, scan_task).await.unwrap_or(Err(ScanError::with_msg("Scan timed out")));
}

struct PamBtBeacon;
pam_hooks!(PamBtBeacon);

impl PamHooks for PamBtBeacon {
    fn sm_authenticate(_pamh: &PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        let settings_path;
        match std::env::var("HOME") {
            Ok(val) => { settings_path = Some(val + "/.pam_btbeacon.yml"); }
            Err(_) => { return PamResultCode::PAM_IGNORE; }
        }
        println!("Reading settings from: {:?}", settings_path);
        let task = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { run(settings_path).await });
        if let Ok(_) = task {
            return PamResultCode::PAM_SUCCESS;
        }
        return PamResultCode::PAM_IGNORE;
    }

    fn sm_setcred(_pamh: &PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        PamResultCode::PAM_SUCCESS
    }

    fn acct_mgmt(_pamh: &PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        println!("account management");
        PamResultCode::PAM_SUCCESS
    }
}
