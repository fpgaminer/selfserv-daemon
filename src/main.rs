use std::time::{Instant, Duration};
use std::net::{UdpSocket, IpAddr, Ipv4Addr};
use structopt::StructOpt;
use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Error};


#[derive(StructOpt)]
struct Cli {
	#[structopt(long = "token", parse(from_os_str))]
	/// Path to a file containing the auth_token
	auth_token_path: PathBuf,

	#[structopt(long = "cert", parse(from_os_str))]
	/// Path to save the SSL certificate to
	cert_path: PathBuf,

	#[structopt(long = "key", parse(from_os_str))]
	/// Path to save the SSL key to
	key_path: PathBuf,

	#[structopt(long = "ip")]
	/// Use this IP address instead of automatically detecting one
	ip_address: Option<Ipv4Addr>,
}


fn main() -> Result<(), Error> {
	let args = Cli::from_args();

	let auth_token = fs::read_to_string(&args.auth_token_path).context("Unable to read auth token")?;
	let auth_token = auth_token.trim();
	let mut previous_ip = None;
	let mut last_update = Instant::now();
	let mut first = true;

	loop {
		if !first {
			std::thread::sleep(Duration::from_secs(60));
		}

		first = false;

		let ip_address = if let Some(addr) = args.ip_address {
			addr
		} else {
			match get_local_ip().context("Could not determine local IP address") {
				Ok(addr) => addr,
				Err(err) => {
					eprintln!("Could not determine local IP address: {}", err);
					continue;
				},
			}
		};

		// Update only if our IP has changed, or if it has been over 24 hours since the last update (to keep our cert up-to-date)
		if last_update.elapsed() < Duration::from_secs(24*60*60) && Some(ip_address) == previous_ip {
			continue;
		}

		println!("Updating: {}", ip_address);

		let (cert, cert_key) = match selfserv(auth_token, &ip_address) {
			Ok(x) => x,
			Err(err) => {
				eprintln!("Error while talking to selfserv.net: {}", err);
				continue;
			}
		};

		fs::write(&args.cert_path, cert).context("Unable to write certificate")?;
		fs::write(&args.key_path, cert_key).context("Unable to write key")?;

		last_update = Instant::now();
		previous_ip = Some(ip_address);
	}
}

// Talks to selfserv.net to update our IP address and returns the SSL certificate and key.
fn selfserv(auth_token: &str, ip_address: &Ipv4Addr) -> Result<(String, String), Error> {
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(70))
		.build()?;
	
	let response = client.post("https://selfserv.net/api/service/ping")
		.header(reqwest::header::AUTHORIZATION, format!("token {}", auth_token))
		.form(&[("ip", ip_address)])
		.send()?;
	
	if !response.status().is_success() {
		return Err(Error::msg(format!("Returned error: {} {:?}", response.status(), response.text())));
	}

	#[derive(Deserialize)]
	struct SelfservResponse {
		cert: String,
		cert_key: String,
	}

	let response: SelfservResponse = response.json()?;

	Ok((response.cert, response.cert_key))
}

// Attempts to determine the local ip address
fn get_local_ip() -> Result<Ipv4Addr, Error> {
	let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("10.255.255.255:1")?;
    let addr = socket.local_addr()?;
	match addr.ip() {
		IpAddr::V4(addr) => Ok(addr),
		_ => Err(Error::msg(format!("IPV6 addresses are not currently supported"))),
	}
}