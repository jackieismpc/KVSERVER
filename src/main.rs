use std::sync::Arc;// 共享引用计数，线程安全

// 提供异步功能的库，AsyncBufReadExt: 异步读取行，AsyncWriteExt: 异步写入数据，BufReader: 缓冲读取器
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// 提供异步网络功能的库，TcpListener: 监听TCP连接，TcpStream: TCP连接
use tokio::net::{TcpListener, TcpStream};
use kvserver::ShardedDb;

type Db = Arc<ShardedDb>;

/// 定义一个枚举，表示支持的命令类型
#[derive(Debug)]
enum Command {
    Get(String),
    Put(String, Vec<u8>),
    Delete(String),
}

/// 解析命令字符串，返回一个Command枚举实例或错误信息
fn parse_command(line: &str) -> Result<Command, String> {
    let mut parts = line.trim().splitn(3, ' ');

    let cmd = parts.next().ok_or("empty command")?;
    match cmd {
        "GET" => {
            let key = parts.next().ok_or("missing key")?;
            Ok(Command::Get(key.to_string()))
        }
        "PUT" => {
            let key = parts.next().ok_or("missing key")?;
            let value = parts.next().ok_or("missing value")?;
            Ok(Command::Put(key.to_string(), value.as_bytes().to_vec()))
        }
        "DELETE" => {
            let key = parts.next().ok_or("missing key")?;
            Ok(Command::Delete(key.to_string()))
        }
        _ => Err(format!("unknown command: {}", cmd)),
    }
}
// 处理客户端连接，读取命令并执行相应的数据库操作
async fn handle_client(stream: TcpStream, db: Db) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();// 将TCP连接分成读写两部分，reader用于读取数据，writer用于写入数据，into_split方法会返回一个元组，包含读写部分的句柄
    let mut reader = BufReader::new(reader);// 创建一个BufReader实例，包装reader句柄，提供缓冲读取功能，可以按行读取数据
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;// 读取一行数据到line字符串中，返回读取的字节数，如果n为0表示连接关闭，跳出循环
        if n == 0 {
            break;
        }

        let response = match parse_command(&line) {
            Ok(Command::Get(key)) => {
                match db.get(&key).await {
                    Some(value) => format!("VALUE {}\n", String::from_utf8_lossy(&value)),// format宏用于格式化字符串，{}表示占位符，String::from_utf8_lossy将字节数组转换为字符串，如果字节数组不是有效的UTF-8编码，则替换为字符
                    None => "NOT_FOUND\n".to_string(),
                }
            }
            Ok(Command::Put(key, value)) => {
                db.put(key, value).await;
                "OK\n".to_string()
            }
            Ok(Command::Delete(key)) => {
                if db.delete(&key).await {
                    "OK\n".to_string()
                } else {
                    "NOT_FOUND\n".to_string()
                }
            }

            Err(e) => format!("ERR {}\n", e),
        };

        writer.write_all(response.as_bytes()).await?;// 将响应字符串转换为字节数组，并异步写入到客户端连接中，write_all方法会确保所有数据都被写入，如果发生错误会返回一个错误
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:12345").await?;
    let db: Db = Arc::new(ShardedDb::new(16));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("accepted connection from {}", addr);

        let db = db.clone();// 克隆数据库的Arc引用计数，使每个客户端连接都能共享同一个数据库实例，clone方法会增加引用计数，但不会复制数据
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, db).await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}