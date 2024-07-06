use std::env;

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName},
};
use tiny_http::{Header, Response, Server};

fn main() {
    // 服务器地址设置
    let server_address = env::var("FORWARD_SERVER").unwrap_or(String::from("127.0.0.1:10086"));
    let server = Server::http(&server_address).unwrap();

    // 远程地址设置
    let forward_address =
        env::var("TARGET_ADDRESS").unwrap_or(String::from("https://api.codemao.cn"));

    println!("Server listening on {}, forwarding {}", server_address, forward_address);
    println!("NOTE: Set the environment variable FORWARD_SERVER to change the server address, and the environment variable TARGET_ADDRESS to change the remote address.");
    // 创建发请求的客户端
    let client = Client::new();

    // 收到的请求计数
    let mut count = 0;

    // 对每个传来的请求
    for mut request in server.incoming_requests() {
        // 请求计数加一
        count += 1;

        // 读取请求头
        let headers = request.headers().iter();

        // 获取UA
        let mut user_agent = String::from("");
        for header in headers.clone() {
            if header.field.equiv("User-Agent") {
                user_agent = header.value.to_string();
            }
        }

        // 获取请求头Map
        let mut headers_map = HeaderMap::new();
        for header in headers.clone() {
            let Header { field, value } = header;
            if field.equiv("Host") {
                continue;
            }
            let name = HeaderName::from_bytes(field.as_str().as_bytes()).unwrap();
            let val = value.to_string().parse().unwrap();
            headers_map.insert(name, val);
        }

        // 获取body
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).unwrap();

        // 打印
        println!(
            "[{}] {:?} {:?}, User-Agent: {:?}",
            count,
            request.method(),
            request.url(),
            user_agent
        );

        // 转换method
        let method = match request.method() {
            tiny_http::Method::Get => reqwest::Method::GET,
            tiny_http::Method::Post => reqwest::Method::POST,
            tiny_http::Method::Put => reqwest::Method::PUT,
            tiny_http::Method::Delete => reqwest::Method::DELETE,
            tiny_http::Method::Head => reqwest::Method::HEAD,
            tiny_http::Method::Options => reqwest::Method::OPTIONS,
            tiny_http::Method::Patch => reqwest::Method::PATCH,
            tiny_http::Method::Connect => reqwest::Method::CONNECT,
            tiny_http::Method::Trace => reqwest::Method::TRACE,
            _ => reqwest::Method::from_bytes(request.method().as_str().as_bytes()).unwrap(),
        };

        // 发送请求
        let req = client
            .request(method, forward_address.clone() + &request.url().to_string())
            .body(body)
            .headers(headers_map)
            .send();

        if let Err(e) = req {
            println!("Error during forwarding: {}", e);
            let resp = Response::from_string(format!("Error during forwarding: {}", e));
            request.respond(resp).unwrap();
            continue;
        }
        if let Ok(msg) = req {
            let resp = Response::new(
                msg.status().as_u16().into(),
                {
                    let mut headers = Vec::new();
                    for (name, value) in msg.headers() {
                        headers.push(Header::from_bytes(name.as_str(), value.as_bytes()).unwrap());
                    }
                    headers.push(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
                    headers
                },
                msg,
                None,
                None,
            );
            request.respond(resp).unwrap();
        }
        
        
    }
}
