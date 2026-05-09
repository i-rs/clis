use clap::{Parser, ValueEnum};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[derive(Parser, Debug)]
#[command(name = "i-rs-http")]
#[command(about = "HTTP client CLI", long_about = None)]
struct Cli {
    #[arg(value_enum, default_value_t = HttpMethod::Get)]
    method: HttpMethod,

    #[arg(value_name = "URL")]
    url: String,

    #[arg(short, long, value_parser = parse_header, value_name = "KEY:VALUE")]
    header: Vec<(String, String)>,
}

#[derive(ValueEnum, Debug, Clone)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

fn parse_header(s: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid header format, expected KEY:VALUE");
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    for (k, v) in &cli.header {
        let key = HeaderName::from_bytes(k.as_bytes())?;
        let val = HeaderValue::from_str(v)?;
        headers.insert(key, val);
    }

    let response = match cli.method {
        HttpMethod::Get => client.get(&cli.url).headers(headers).send().await?,
        HttpMethod::Post => client.post(&cli.url).headers(headers).send().await?,
        HttpMethod::Put => client.put(&cli.url).headers(headers).send().await?,
        HttpMethod::Delete => client.delete(&cli.url).headers(headers).send().await?,
    };

    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());

    let body = response.text().await?;
    println!("Body:\n{}", body);

    Ok(())
}
