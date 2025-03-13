use anyhow::Result;
use nom::{
    bytes::{self, complete::tag},
    combinator::{flat_map, map, map_res},
    number,
    sequence::{preceded, tuple},
    IResult,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::{net::UdpSocket, time::timeout};

pub async fn discover() -> Result<HashMap<String, String>> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.set_broadcast(true)?;
    let response = timeout(Duration::from_secs(5), async {
        let message = "eNAME\0JSON\0UUID\0VERS\0".as_bytes();
        let mut buf = vec![0u8; 1024];
        let _ = sock.send_to(&message, "255.255.255.255:3483").await?;
        let result = sock.recv_from(&mut buf).await?;
        let ip = result.1.ip().to_string();
        let port = parse_reply(&buf).unwrap().1.to_string();
        //let server = format!("{}:{}", ip, port).to_string();
        let mut server: HashMap<String, String> = HashMap::new();
        server.insert("host".to_string(), ip);
        server.insert("port".to_string(), port);
        Ok::<HashMap<String, String>, anyhow::Error>(server)
    }).await;
    Ok(response.unwrap_or(Result::Ok(HashMap::new())).unwrap())
}

fn parse_tag<'a>(input: &'a [u8], start_tag: &str) -> IResult<&'a [u8], String> {
    map_res(
        preceded(
            tag(start_tag),
            flat_map(number::complete::be_u8, bytes::complete::take),
        ),
        |bytes: &[u8]| String::from_utf8(bytes.to_vec()),
    )(input)
}

fn parse_hostname(input: &[u8]) -> IResult<&[u8], String> {
    parse_tag(input, "ENAME")
}

fn parse_port(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(
        |input| parse_tag(input, "JSON"),
        |s| s.parse::<u16>()
    )(input)
}

fn parse_reply(input: &[u8]) -> IResult<&[u8], u16> {
    map(
        tuple((parse_hostname, parse_port)),
        |(_, port)| 
            port,
    )(input)
}