use std::str::FromStr;
use std::error::Error;
use bluer::{Session, Adapter, Address};

mod settings;

async fn scan_for_target(adapter: &Adapter, target: &str) -> Result<bool, Box<dyn Error>> {
    let target_address: Address = Address::from_str(target)?;
    let discovery_stream = adapter.discover_devices().await?;
    let discovered = adapter.device_addresses().await?;
    println!("Discovered devices: {:#?}", discovered);
    if discovered.contains(&target_address) {
        return Ok(true);
    }
    Ok(false)
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

async fn run() -> Result<(), Box<dyn Error>> {
    settings::load_settings("./settings.yml")?;
    let s = settings::get();
    let adapter = init_adapter().await?;
    println!("Scan result: {:?}", scan_for_target(&adapter, &s.target).await);
    Ok(())
}

#[tokio::main]
async fn main() {
    let task = tokio::task::spawn(async move {
        run().await.unwrap();
    });
    task.await.unwrap();
}
